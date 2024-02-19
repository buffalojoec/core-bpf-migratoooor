//! No-op program for feature gate

use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    program::set_return_data,
    pubkey::Pubkey,
};

entrypoint!(noop);

pub fn noop(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    set_return_data(&[7; 32]);
    Ok(())
}
