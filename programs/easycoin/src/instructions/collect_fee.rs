use anchor_lang::{
    prelude::*,
    solana_program::sysvar::{self},
    system_program,
};

use crate::errors::*;
use crate::state::*;
use crate::events::*;


#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct CollectFeeArgs {
    pub user_account_nonce: u32,
    pub only_trade_fee: bool,
}

#[derive(Accounts)]
#[instruction(args: CollectFeeArgs)]
pub struct CollectFee<'info> {
    #[account(
        mut,
        seeds = [SEED_PREFIX, SEED_USER, owner_account.key().as_ref(), &args.user_account_nonce.to_le_bytes()],
        bump
    )]
    pub user_account: SystemAccount<'info>, // PDA
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
        mut,
        constraint = fee_registry.is_fee_collector(tx_fee_collector.key()) @ AgentError::FeeCollectorInvalid
    )]
    pub tx_fee_collector: SystemAccount<'info>,
    #[account(
        mut,
        constraint = fee_registry.is_fee_collector(trade_fee_collector.key()) @ AgentError::FeeCollectorInvalid
    )]
    pub trade_fee_collector: SystemAccount<'info>,
    #[account(
        seeds = [SEED_PREFIX, SEED_FEE],
        bump
    )]
    pub fee_registry: Account<'info, FeeRegistry>,
    #[account(
        address = sysvar::instructions::id() // instructions sysvar
    )]
    /// CHECK: instructions_sysvar account
    pub instructions_sysvar: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl CollectFee<'_> {
    pub fn collect_fee(ctx: Context<CollectFee>, args: CollectFeeArgs) -> Result<()> {

        require!(ctx.accounts.operator_registry.program_ok(), AgentError::ProgramStatusError);

        let mut tx_fee = 0;
        if !args.only_trade_fee {
            tx_fee = FeeRegistry::calculate_tx_fee(
                ctx.accounts.instructions_sysvar.to_account_info().as_ref(),
            )?;
        }

        let owner_account = &mut ctx.accounts.owner_account;
        let due_swap_fee = owner_account.get_user_account_due_fee(args.user_account_nonce)?;
        owner_account.sub_user_account_due_fee(args.user_account_nonce, due_swap_fee)?;

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

        #[cfg(feature = "enable-log")]
        msg!("collect_fee: tx_fee={} due_fee={}", tx_fee, due_swap_fee);

        // collect trade fee
        if due_swap_fee > 0 {
            system_program::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.system_program.to_account_info(),
                    system_program::Transfer {
                        from: ctx.accounts.user_account.to_account_info(),
                        to: ctx.accounts.trade_fee_collector.to_account_info(),
                    },
                    signer_seeds,
                ),
                due_swap_fee,
            )?;
        }

        if tx_fee > 0 {
            // collect tx fee
            system_program::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.system_program.to_account_info(),
                    system_program::Transfer {
                        from: ctx.accounts.user_account.to_account_info(),
                        to: ctx.accounts.tx_fee_collector.to_account_info(),
                    },
                    signer_seeds,
                ),
                tx_fee,
            )?;
        }

        emit!(FeeCollectedEvent {
            user_account: ctx.accounts.user_account.key(),
            tx_fee,
            trade_fee: due_swap_fee,
        });

        Ok(())
    }
}
