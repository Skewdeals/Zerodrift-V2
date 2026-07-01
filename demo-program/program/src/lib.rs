//! Minimal, from-scratch demo program: one instruction whose cost scales
//! with a count `n`, built to show the affine-CU-envelope technique against
//! real, compiled BPF bytecode instead of illustrative numbers.
//!
//! Wire format: instruction data = n:u32 LE, followed by n * u64 LE items.
//! State account: 12 bytes = accumulator:u64 LE (offset 0) + last_n:u32 LE (offset 8).

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

entrypoint!(process_instruction);

const FOLD_CONST: u64 = 2654435761;

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let state_account = next_account_info(account_info_iter)?;

    if instruction_data.len() < 4 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let n = u32::from_le_bytes(instruction_data[0..4].try_into().unwrap()) as usize;

    let expected_len = 4 + n * 8;
    if instruction_data.len() != expected_len {
        return Err(ProgramError::InvalidInstructionData);
    }

    let mut acc: u64 = 0;
    for i in 0..n {
        let start = 4 + i * 8;
        let item = u64::from_le_bytes(instruction_data[start..start + 8].try_into().unwrap());
        acc = acc.wrapping_add(item).wrapping_mul(FOLD_CONST);
    }

    let mut data = state_account.try_borrow_mut_data()?;
    if data.len() < 12 {
        return Err(ProgramError::AccountDataTooSmall);
    }
    data[0..8].copy_from_slice(&acc.to_le_bytes());
    data[8..12].copy_from_slice(&(n as u32).to_le_bytes());

    Ok(())
}
