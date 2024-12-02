//! A simple stub program.
//!
//! This program's ELF is used when running a stub test, where the program's
//! buffer account contains this simple program.
//!
//! The program is designed to be deterministic, to allow for the same test
//! suite to be used across different migrations.
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    incinerator,
    instruction::{AccountMeta, Instruction},
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction, system_program,
    sysvar::Sysvar,
};

#[cfg(feature = "sbf-entrypoint")]
solana_program::entrypoint!(process);

pub fn write(
    program_id: &Pubkey,
    target_address: &Pubkey,
    payer_address: &Pubkey,
    data: &[u8],
) -> Instruction {
    let mut input = vec![0; data.len() + 1];
    input[1..].copy_from_slice(data);
    Instruction::new_with_bytes(
        *program_id,
        &input,
        vec![
            AccountMeta::new(*target_address, true),
            AccountMeta::new(*payer_address, true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    )
}

pub fn burn(program_id: &Pubkey, target_address: &Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        *program_id,
        &[1],
        vec![
            AccountMeta::new(*target_address, true),
            AccountMeta::new(incinerator::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    )
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    match input.split_first() {
        Some((&0, rest)) => {
            // Write:
            // * Fund the target account to rent-exemption.
            // * Allocate space for the input data.
            // * Assign ownership to the program.
            // * Write the input data into it.
            let accounts_iter = &mut accounts.iter();
            let target_info = next_account_info(accounts_iter)?;
            let payer_info = next_account_info(accounts_iter)?;
            let _system_program_info = next_account_info(accounts_iter)?;

            if !payer_info.is_signer {
                Err(ProgramError::MissingRequiredSignature)?
            }

            let rent = <Rent as Sysvar>::get()?;
            let lamports = rent.minimum_balance(rest.len());

            invoke(
                &system_instruction::transfer(payer_info.key, target_info.key, lamports),
                &[payer_info.clone(), target_info.clone()],
            )?;
            invoke(
                &system_instruction::allocate(target_info.key, rest.len() as u64),
                &[target_info.clone()],
            )?;
            invoke(
                &system_instruction::assign(target_info.key, program_id),
                &[target_info.clone()],
            )?;

            let mut data = target_info.try_borrow_mut_data()?;
            data[..].copy_from_slice(rest);

            Ok(())
        }
        Some((&1, _)) => {
            // Burn:
            // * Burn all of the lamports in the target account.
            let accounts_iter = &mut accounts.iter();
            let target_info = next_account_info(accounts_iter)?;
            let incinerator_info = next_account_info(accounts_iter)?;
            let _system_program_info = next_account_info(accounts_iter)?;

            if !target_info.is_signer {
                Err(ProgramError::MissingRequiredSignature)?
            }

            invoke(
                &system_instruction::transfer(
                    target_info.key,
                    incinerator_info.key,
                    target_info.lamports(),
                ),
                &[target_info.clone(), incinerator_info.clone()],
            )?;

            Ok(())
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
