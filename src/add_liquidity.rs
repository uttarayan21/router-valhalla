//! Add liquidity to valhalla

// use solana_farm_sdk::program::account;
use crate::utils::{self, validate};
use solana_farm_sdk::program::account;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
};
use std::convert::TryFrom;

// fn validate_accounts<'a, 'b>(
//     accounts: &'a [AccountInfo<'b>],
// ) -> Result<&'a [AccountInfo<'b>; 9], ProgramError> {
//     if let Ok(accounts) = <&[AccountInfo; 9]>::try_from(accounts) {
//         Ok(accounts)
//     } else {
//         Err(ProgramError::NotEnoughAccountKeys)
//     }
// }

pub fn add_liquidity(
    accounts: &[AccountInfo],
    max_token_a_amount: u64,
    max_token_b_amount: u64,
) -> ProgramResult {
    msg!("Processing AmmInstruction::AddLiquidity");
    msg!("max_token_a_amount {} ", max_token_a_amount);
    msg!("max_token_b_amount {} ", max_token_b_amount);

    //   0. `[]` Token-swap
    //   1. `[]` swap authority
    //   2. `[]` user transfer authority
    //   3. `[writable]` token_a user transfer authority can transfer amount,
    //   4. `[writable]` token_b user transfer authority can transfer amount,
    //   5. `[writable]` token_a Base Account to deposit into.
    //   6. `[writable]` token_b Base Account to deposit into.
    //   7. `[writable]` Pool MINT account, swap authority is the owner.
    //   8. `[writable]` Pool Account to deposit the generated tokens, user is the owner.
    //   9. `[]` Token program id

    #[rustfmt::skip]
    let [
        user_account,
        user_token_a_account,
        user_token_b_account,
        user_lp_token_account,
        pool_program_id,
        pool_token_a_account,
        pool_token_b_account,
        lp_token_mint,
        _spl_token_id,
        _clock_id,
        swap_account,
        swap_authority,
    ] =
        validate!(accounts, 12);

    let (lp_token_amount, token_a_amount, token_b_amount) = utils::get_pool_deposit_amounts(
        pool_token_a_account,
        pool_token_b_account,
        lp_token_mint,
        max_token_a_amount,
        max_token_b_amount,
    )?;

    let initial_token_a_user_balance = account::get_token_balance(user_token_a_account)?;
    let initial_token_b_user_balance = account::get_token_balance(user_token_b_account)?;
    let initial_lp_token_user_balance = account::get_token_balance(user_lp_token_account)?;

    // We need to call DepositAllTokenTypes for add_liquidity function
    // Or DepositSingleTokenTypeExactAmountIn for adding a single token

    let data = spl_token_swap::instruction::DepositAllTokenTypes {
        // spl_token_swap::instruction::DepositAllTokenTypes {
        pool_token_amount: lp_token_amount,
        maximum_token_a_amount: token_a_amount,
        maximum_token_b_amount: token_b_amount,
    };

    let instruction = spl_token_swap::instruction::deposit_all_token_types(
        pool_program_id.key,
        &spl_token::id(),
        swap_account.key,
        swap_authority.key,
        user_account.key,
        user_token_a_account.key,
        user_token_b_account.key,
        pool_token_a_account.key,
        pool_token_b_account.key,
        lp_token_mint.key,
        user_lp_token_account.key,
        data,
    )?;

    // let token_swap = SwapVersion::unpack(&swap_info.data.borrow())?;
    // let calculator = &token_swap.swap_curve().calculator;

    solana_program::program::invoke(&instruction, accounts)?;

    account::check_tokens_spent(
        user_token_a_account,
        initial_token_a_user_balance,
        max_token_a_amount,
    )?;

    account::check_tokens_spent(
        user_token_b_account,
        initial_token_b_user_balance,
        max_token_b_amount,
    )?;

    account::check_tokens_received(user_lp_token_account, initial_lp_token_user_balance, 1)?;
    msg!("AmmInstruction::AddLiquidity complete");
    Ok(())
}
