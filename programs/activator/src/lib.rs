//! Program to facilitate creation of feature accounts without a keypair.
//!
//! Core contributors may wish to test Core BPF migration feature activations,
//! but they may not be the keypair holder for the particular target feature.
//!
//! The test harness will create a feature account owned by this program at
//! genesis. Then, it can invoke this program to assign ownership to
//! `Feature1111...`, activating the feature without the keypair.
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
};

const FEATURE_GATE_PROGRAM_ID: Pubkey =
    solana_program::pubkey!("Feature111111111111111111111111111111111111");

solana_program::declare_id!("CBMActivator1111111111111111111111111111111");

solana_program::entrypoint!(process);

pub fn activate_feature(feature_id: &Pubkey) -> Instruction {
    Instruction::new_with_bytes(crate::id(), &[], vec![AccountMeta::new(*feature_id, false)])
}

pub fn process(_program_id: &Pubkey, accounts: &[AccountInfo], _input: &[u8]) -> ProgramResult {
    accounts
        .first()
        .ok_or(ProgramError::NotEnoughAccountKeys)
        .map(|info| info.assign(&FEATURE_GATE_PROGRAM_ID))
}
