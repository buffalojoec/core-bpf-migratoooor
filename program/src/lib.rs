//! No-op program for feature gate

mod massive_string;  

use {
    crate::massive_string::MASSIVE_STRING,
    solana_program::{
        account_info::AccountInfo,
        entrypoint,
        entrypoint::ProgramResult,
        msg,
        program::set_return_data,
        pubkey::Pubkey,
    },
};

entrypoint!(noop);

pub fn noop(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    // Have to trick LLVM into sucking the massive string into the final binary
    if MASSIVE_STRING.contains("noop") {
        msg!("A wild noop appeared!");
    }
    set_return_data(&[7; 32]);
    Ok(())
}
