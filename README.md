# Valhalla router

Trying to implement a router like raydium/saber for token-swap

1. [x] add_liquidity -> [spl_token_swap::instruction::DepositAllTokenTypes](https://docs.rs/spl-token-swap/latest/spl_token_swap/instruction/struct.DepositAllTokenTypes.html)
2. [x] remove_liquidity -> [spl_token_swap::instruction::WithdrawAllTokenTypes](https://docs.rs/spl-token-swap/latest/spl_token_swap/instruction/struct.WithdrawAllTokenTypes.html)
3. [x] swap -> [spl_token_swap::instruction::Swap](https://docs.rs/spl-token-swap/latest/spl_token_swap/instruction/struct.Swap.html)
4. [ ] stake
5. [ ] unstake
6. [ ] harvest

We need to implement a token-farm for staking token-farm lp tokens.
I tried using the [quarry](https://github.com/QuarryProtocol/quarry) from QuarryProtocol but I don't have enough js knowledge to make the SDK work.

Currently working on a token-farm which will give stake unstake and harvest functions.

