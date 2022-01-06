use solana_farm_sdk::{math, program::account};
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError};

pub const VALHALLA_FEE: f64 = 0.003;

/// This will return the validated value if the number of keys matched else it will return Err(ProgramError::NotEnoughAccountKeys)
macro_rules! validate {
    ($accounts:expr, $num:expr) => {
        match <&[AccountInfo; $num]>::try_from($accounts) {
            Ok(accounts) => accounts,
            Err(_) => {
                msg!("Not enough keys were supplied");
                return Err(ProgramError::NotEnoughAccountKeys);
            }
        }
    };
}
pub(crate) use validate;

pub fn get_pool_withdrawal_amounts<'a, 'b>(
    pool_token_a_account: &'a AccountInfo<'b>,
    pool_token_b_account: &'a AccountInfo<'b>,
    lp_token_mint: &'a AccountInfo<'b>,
    lp_token_amount: u64,
) -> Result<(u64, u64), ProgramError> {
    if lp_token_amount == 0 {
        msg!("Error: LP token amount must be non-zero");
        return Err(ProgramError::InvalidArgument);
    }
    let (token_a_balance, token_b_balance) =
        get_pool_token_balances(pool_token_a_account, pool_token_b_account)?;

    if token_a_balance == 0 && token_b_balance == 0 {
        return Ok((0, 0));
    }
    let lp_token_supply = account::get_token_supply(lp_token_mint)?;
    if lp_token_supply == 0 {
        return Ok((0, 0));
    }
    // WARN: Possible bug ?
    // u64::MAX        = 18446744073709551615
    // u64::MAX as f64 = 18446744073709552000.0
    // Which is greater than the previous number
    let stake = lp_token_amount as f64 / lp_token_supply as f64;

    Ok((
        math::checked_as_u64(token_a_balance as f64 * stake)?,
        math::checked_as_u64(token_b_balance as f64 * stake)?,
    ))
}

pub fn get_pool_token_balances<'a, 'b>(
    pool_token_a_account: &'a AccountInfo<'b>,
    pool_token_b_account: &'a AccountInfo<'b>,
) -> Result<(u64, u64), ProgramError> {
    Ok((
        account::get_token_balance(pool_token_a_account)?,
        account::get_token_balance(pool_token_b_account)?,
    ))
}

pub fn get_pool_deposit_amounts<'a, 'b>(
    pool_token_a_account: &'a AccountInfo<'b>,
    pool_token_b_account: &'a AccountInfo<'b>,
    lp_token_mint: &'a AccountInfo<'b>,
    max_token_a_amount: u64,
    max_token_b_amount: u64,
) -> Result<(u64, u64, u64), ProgramError> {
    if max_token_a_amount == 0 && max_token_b_amount == 0 {
        msg!("Error: At least one of token amounts must be non-zero");
        return Err(ProgramError::InvalidArgument);
    }
    let mut token_a_amount = max_token_a_amount;
    let mut token_b_amount = max_token_b_amount;
    let (token_a_balance, token_b_balance) =
        get_pool_token_balances(pool_token_a_account, pool_token_b_account)?;

    if token_a_balance == 0 || token_b_balance == 0 {
        if max_token_a_amount == 0 || max_token_b_amount == 0 {
            msg!("Error: Both amounts must be specified for the initial deposit to an empty pool");
            return Err(ProgramError::InvalidArgument);
        } else {
            return Ok((1, max_token_a_amount, max_token_b_amount));
        }
    }

    if max_token_a_amount == 0 {
        let estimated_coin_amount = math::checked_as_u64(
            token_a_balance as f64 * max_token_b_amount as f64 / (token_b_balance as f64),
        )?;
        token_a_amount = if estimated_coin_amount > 1 {
            estimated_coin_amount - 1
        } else {
            0
        };
    } else if max_token_b_amount == 0 {
        token_b_amount = math::checked_as_u64(
            token_b_balance as f64 * max_token_a_amount as f64 / (token_a_balance as f64),
        )?;
    }

    let min_lp_tokens_out = estimate_lp_tokens_amount(
        lp_token_mint,
        token_a_amount,
        token_b_amount,
        token_a_balance,
        token_b_balance,
    )?;

    Ok((
        min_lp_tokens_out,
        token_a_amount,
        math::checked_add(token_b_amount, 1)?,
    ))
}

pub fn estimate_lp_tokens_amount(
    lp_token_mint: &AccountInfo,
    token_a_deposit: u64,
    token_b_deposit: u64,
    pool_token_a_balance: u64,
    pool_token_b_balance: u64,
) -> Result<u64, ProgramError> {
    if pool_token_a_balance != 0 && pool_token_b_balance != 0 {
        Ok(std::cmp::min(
            math::checked_as_u64(
                (token_a_deposit as f64 / pool_token_a_balance as f64)
                    * account::get_token_supply(lp_token_mint)? as f64,
            )?,
            math::checked_as_u64(
                (token_b_deposit as f64 / pool_token_b_balance as f64)
                    * account::get_token_supply(lp_token_mint)? as f64,
            )?,
        ))
    } else if pool_token_a_balance != 0 {
        math::checked_as_u64(
            (token_a_deposit as f64 / pool_token_a_balance as f64)
                * account::get_token_supply(lp_token_mint)? as f64,
        )
    } else if pool_token_b_balance != 0 {
        math::checked_as_u64(
            (token_b_deposit as f64 / pool_token_b_balance as f64)
                * account::get_token_supply(lp_token_mint)? as f64,
        )
    } else {
        Ok(0)
    }
}

pub fn get_pool_swap_amounts<'a, 'b>(
    pool_token_a_account: &'a AccountInfo<'b>,
    pool_token_b_account: &'a AccountInfo<'b>,
    token_a_amount_in: u64,
    token_b_amount_in: u64,
) -> Result<(u64, u64), ProgramError> {
    if (token_a_amount_in == 0 && token_b_amount_in == 0)
        || (token_a_amount_in > 0 && token_b_amount_in > 0)
    {
        msg!("Error: One and only one of token amounts must be non-zero");
        return Err(ProgramError::InvalidArgument);
    }
    let (token_a_balance, token_b_balance) =
        get_pool_token_balances(pool_token_a_account, pool_token_b_account)?;
    if token_a_balance == 0 || token_b_balance == 0 {
        msg!("Error: Can't swap in an empty pool");
        return Err(ProgramError::Custom(412));
    }
    let token_a_balance = token_a_balance as f64;
    let token_b_balance = token_b_balance as f64;
    if token_a_amount_in == 0 {
        // b to a
        let amount_in_no_fee = ((token_b_amount_in as f64 * (1.0 - VALHALLA_FEE)) as u64) as f64;
        let estimated_token_a_amount = (token_a_balance
            - token_a_balance * token_b_balance / (token_b_balance + amount_in_no_fee))
            as u64;

        Ok((token_b_amount_in, estimated_token_a_amount))
    } else {
        // a to b
        let amount_in_no_fee = ((token_a_amount_in as f64 * (1.0 - VALHALLA_FEE)) as u64) as f64;
        let estimated_token_b_amount = (token_b_balance
            - token_a_balance * token_b_balance / (token_a_balance + amount_in_no_fee))
            as u64;

        Ok((token_a_amount_in, estimated_token_b_amount))
    }
}
