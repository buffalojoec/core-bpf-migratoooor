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
    instruction::{AccountMeta, Instruction},
    program::{invoke, set_return_data},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction, system_program,
    sysvar::Sysvar,
};

solana_program::declare_id!("CBMStub111111111111111111111111111111111111");

solana_program::entrypoint!(process);

pub fn write(target_address: &Pubkey, payer_address: &Pubkey, data: &[u8]) -> Instruction {
    let mut input = Vec::with_capacity(data.len() + 1);
    input[1..].copy_from_slice(data);
    Instruction::new_with_bytes(
        crate::id(),
        &input,
        vec![
            AccountMeta::new(*target_address, false),
            AccountMeta::new(*payer_address, true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    )
}

pub fn emit(data: &[u8]) -> Instruction {
    let mut input = Vec::with_capacity(data.len() + 1);
    input[0] = 1;
    input[1..].copy_from_slice(data);
    Instruction::new_with_bytes(crate::id(), &input, Vec::default())
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
        Some((&1, rest)) => {
            // Emit:
            // * Set the return data to the provided input.
            set_return_data(rest);

            Ok(())
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
