//! Swap tokens with the Orca pool instruction

use crate::utils::{self, validate};
// use solana_farm_sdk::program::account;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke,
    program_error::ProgramError,
};

pub fn swap(
    accounts: &[AccountInfo],
    token_a_in: u64,
    token_b_in: u64,
    min_token_amount_out: u64,
) -> ProgramResult {
    msg!("Processing Ammspl_token_swap::instruction::Swap");

    #[rustfmt::skip]
    let [
        user_account,
        user_token_a_account,
        user_token_b_account,
        pool_program_id,
        pool_token_a_account,
        pool_token_b_account,
        lp_token_mint,
        _spl_token_id,
        amm_id,
        amm_authority,
        fees_account
        ] = validate!(accounts, 11);

    let (amount_in, mut minimum_amount_out) = utils::get_pool_swap_amounts(
        pool_token_a_account,
        pool_token_b_account,
        token_a_in,
        token_b_in,
    )?;

    if min_token_amount_out > minimum_amount_out {
        minimum_amount_out = min_token_amount_out
    }

    let data = spl_token_swap::instruction::Swap {
        amount_in,
        minimum_amount_out,
    };

    if token_a_in == 0 {
        let instruction = spl_token_swap::instruction::swap(
            pool_program_id.key,
            &spl_token::id(),
            amm_id.key,
            amm_authority.key,
            user_account.key,
            user_token_b_account.key,
            pool_token_b_account.key,
            pool_token_a_account.key,
            user_token_a_account.key,
            lp_token_mint.key,
            fees_account.key,
            None,
            data,
        )?;
        invoke(&instruction, accounts)?;
    } else {
        let instruction = spl_token_swap::instruction::swap(
            pool_program_id.key,
            &spl_token::id(),
            amm_id.key,
            amm_authority.key,
            user_account.key,
            user_token_a_account.key,
            pool_token_a_account.key,
            pool_token_b_account.key,
            user_token_b_account.key,
            lp_token_mint.key,
            fees_account.key,
            None,
            data,
        )?;
        invoke(&instruction, accounts)?;
    }

    msg!("Ammspl_token_swap::instruction::Swap complete");
    Ok(())
}
