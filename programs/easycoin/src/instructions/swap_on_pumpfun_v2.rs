use std::u64;

use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke_signed},
};
use anchor_spl::token_interface::TokenAccount;

use crate::errors::*;
use crate::external_program::Pumpfun;
use crate::state::*;
use pumpfun_cpi;

#[account]
#[derive(Debug)]
struct BondingCurve {
    pub virtual_token_reserves: u64,
    pub virtual_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub token_total_supply: u64,
    pub complete: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum SwapOnPumpfunV2Op {
    Buy {
        sol_amount: u64,
        min_token_output: u64,
    },
    Sell {
        token_amount: u64,
        min_sol_output: u64,
    },
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct SwapOnPumpfunV2Args {
    user_account_nonce: u32,
    op: SwapOnPumpfunV2Op,
}

#[derive(Accounts)]
#[instruction(args: SwapOnPumpfunV2Args)]
pub struct SwapOnPumpfunV2<'info> {
    #[account(
        mut,
        seeds = [SEED_PREFIX, SEED_USER, owner_account.key().as_ref(), &args.user_account_nonce.to_le_bytes()],
        bump
    )]
    pub user_account: SystemAccount<'info>,
    #[account(mut)]
    pub owner_account: Account<'info, OwnerAccount>,
    #[account(
        constraint = operator_registry.is_operator(operator.key()) @ AgentError::OperatorNotAllowed
    )]
    pub operator: Signer<'info>,
    #[account(
        seeds = [SEED_PREFIX, SEED_OPERATOR],
        bump
    )]
    pub operator_registry: Account<'info, OperatorRegistry>,
    #[account(
        seeds = [SEED_PREFIX, SEED_FEE],
        bump
    )]
    pub fee_registry: Account<'info, FeeRegistry>,
    pub pumpfun_program: Program<'info, Pumpfun>,
}

impl<'info> SwapOnPumpfunV2<'info> {
    fn validate_user_token_account(
        user_account: SystemAccount,
        remaining_accounts: &'info [AccountInfo<'info>],
    ) -> Result<()> {
        // validate user account
        let user_account_info = remaining_accounts
            .get(6)
            .ok_or(AgentError::NotPumpfunTrade)?;
        require!(
            user_account_info.key() == user_account.key(),
            AgentError::PumpfunUserAccountInvalid
        );

        // validate user token account
        let user_token_account_info = remaining_accounts
            .get(5)
            .ok_or(AgentError::NotPumpfunTrade)?;
        let user_token_account: InterfaceAccount<'info, TokenAccount> =
            InterfaceAccount::try_from(user_token_account_info)?;
        require!(
            user_token_account.owner == user_account.key(),
            AgentError::PumpfunUserTokenAccountInvalid
        );

        Ok(())
    }

    pub fn swap_on_pumpfun_v2(
        ctx: Context<'_, '_, 'info, 'info, SwapOnPumpfunV2>,
        args: SwapOnPumpfunV2Args,
    ) -> Result<()> {
        require!(
            ctx.accounts.operator_registry.program_ok(),
            AgentError::ProgramStatusError
        );

        let remaining_accounts = ctx.remaining_accounts;
        let user_account = &ctx.accounts.user_account;

        Self::validate_user_token_account(user_account.clone(), remaining_accounts)?;

        #[cfg(feature = "enable-log")]
        msg!("user token account is valid");

        let bonding_curve_account_info = remaining_accounts
            .get(3)
            .ok_or(AgentError::NotPumpfunTrade)?;
        let bonding_curve = BondingCurve::try_deserialize(
            &mut bonding_curve_account_info.data.borrow()[..].as_ref(),
        )?;

        #[cfg(feature = "enable-log")]
        msg!(
            "bonding_curve {}: {:?}",
            bonding_curve_account_info.key(),
            bonding_curve
        );

        // get user account lamports before
        let user_account_lamports_before = user_account.lamports();
        #[cfg(feature = "enable-log")]
        msg!(
            "user_account_lamports before: {}",
            user_account_lamports_before
        );

        let data = match args.op {
            SwapOnPumpfunV2Op::Buy {
                sol_amount,
                min_token_output,
            } => {

                let sol_amount_minus_fee = sol_amount
                    - sol_amount
                        .checked_div(100)
                        .ok_or(AgentError::SwapAmountOverflow)?; // floor div

                let sol_amount_minus_fee_u128 = u128::from(sol_amount_minus_fee);

                let divisor = u128::from(bonding_curve.virtual_sol_reserves)
                    .checked_add(sol_amount_minus_fee_u128)
                    .ok_or(AgentError::SwapAmountOverflow)?;

                let dividend = u128::from(bonding_curve.virtual_token_reserves)
                    .checked_mul(sol_amount_minus_fee_u128)
                    .ok_or(AgentError::SwapAmountOverflow)?;

                let amount_u128 = dividend
                    .checked_div(divisor)
                    .ok_or(AgentError::SwapAmountOverflow)?;

                #[cfg(feature = "enable-log")]
                msg!(
                    "amount: {}, amount_bk: {}, min_amount_out: {},  sol_amount: {}, sol_amount_minus_fee: {}",
                    amount_u128,
                    amount_u128_bk,
                    min_token_output,
                    sol_amount,
                    sol_amount_minus_fee
                );

                require!(
                    amount_u128 <= u128::from(u64::MAX),
                    AgentError::SwapAmountOverflow
                );
                require!(
                    amount_u128 >= u128::from(min_token_output),
                    AgentError::PumpfunExceedSlippage
                );

                let amount = amount_u128 as u64;

                let ix = pumpfun_cpi::instruction::Buy {
                    _amount: amount,
                    _max_sol_cost: sol_amount + 5, // add 5 in case of rounding issue
                };
                let mut ix_data = Vec::with_capacity(256);
                ix_data.extend_from_slice(&[102, 6, 61, 18, 1, 218, 235, 234]);
                AnchorSerialize::serialize(&ix, &mut ix_data)
                    .map_err(|_| anchor_lang::error::ErrorCode::InstructionDidNotSerialize)?;
                ix_data
            }
            SwapOnPumpfunV2Op::Sell {
                token_amount,
                min_sol_output,
            } => {
                let ix = pumpfun_cpi::instruction::Sell {
                    _amount: token_amount,
                    _min_sol_output: min_sol_output,
                };

                #[cfg(feature = "enable-log")]
                msg!(
                    "token_amount: {}  min_sol_output: {}",
                    token_amount,
                    min_sol_output
                );

                let mut ix_data = Vec::with_capacity(256);
                ix_data.extend_from_slice(&[51, 230, 133, 164, 1, 127, 131, 173]);
                AnchorSerialize::serialize(&ix, &mut ix_data)
                    .map_err(|_| anchor_lang::error::ErrorCode::InstructionDidNotSerialize)?;
                ix_data
            }
        };

        let mut accounts: Vec<AccountMeta> = remaining_accounts
            .iter()
            .map(|acc| AccountMeta {
                pubkey: *acc.key,
                is_signer: acc.is_signer,
                is_writable: acc.is_writable,
            })
            .collect();

        if let Some(signer_account) = accounts.get_mut(6) {
            signer_account.is_signer = true;
        }

        let accounts_infos: Vec<AccountInfo> = remaining_accounts
            .iter()
            .map(|acc| AccountInfo { ..acc.clone() })
            .collect();

        let owner_account_key = ctx.accounts.owner_account.key();
        let user_account_nonce_bytes = args.user_account_nonce.to_le_bytes();
        let user_account_bump_bytes = ctx.bumps.user_account.to_le_bytes();
        let signer_seeds: &[&[&[u8]]] = &[&[
            SEED_PREFIX,
            SEED_USER,
            owner_account_key.as_ref(),
            user_account_nonce_bytes.as_ref(),
            user_account_bump_bytes.as_ref(),
        ]];

        invoke_signed(
            &Instruction {
                program_id: ctx.accounts.pumpfun_program.key(),
                accounts,
                data,
            },
            &accounts_infos,
            signer_seeds,
        )?;

        let user_account_lamports_after = user_account.lamports();
        let trade_fee = ctx
            .accounts
            .fee_registry
            .calculate_swap_fee(user_account_lamports_before, user_account_lamports_after)?;

        // record swap fee
        ctx.accounts
            .owner_account
            .add_user_account_due_fee(args.user_account_nonce, trade_fee)?;

        let required_rent: u64 = Rent::get()?.minimum_balance(user_account.data_len());
        let user_account_balance = user_account.lamports() - required_rent;

        let due_fee = ctx
            .accounts
            .owner_account
            .get_user_account_due_fee(args.user_account_nonce)?;
        require!(
            user_account_balance >= due_fee,
            AgentError::UserAccountBalanceNotEnough
        );

        Ok(())
    }
}
