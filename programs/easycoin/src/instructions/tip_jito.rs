use anchor_lang::{prelude::*, system_program};

use crate::state::*;
use crate::errors::*;
use crate::events::*;

use crate::external_program::JitoTipProgram;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct TipJitoArgs {
    pub user_account_nonce: u32,
    pub tip_amount: u64,
}

#[derive(Accounts)]
#[instruction(args: TipJitoArgs)]
pub struct TipJito<'info> {
    #[account(
        mut,
        seeds = [SEED_PREFIX, SEED_USER, owner_account.key().as_ref(), &args.user_account_nonce.to_le_bytes()],
        bump
    )]
    pub user_account: SystemAccount<'info>, // PDA
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
        owner = JitoTipProgram::id()
    )]
    /// CHECK: Jito tip account is owned by Jito tip program
    pub jito_tip_account: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl TipJito<'_> {
    pub fn tip_jito(ctx: Context<TipJito>, args: TipJitoArgs) -> Result<()> {

        require!(ctx.accounts.operator_registry.program_ok(), AgentError::ProgramStatusError);

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

        system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.user_account.to_account_info(),
                    to: ctx.accounts.jito_tip_account.to_account_info(),
                },
                signer_seeds,
            ),
            args.tip_amount,
        )?;

        emit!(TipJitoEvent {
            user_account: ctx.accounts.user_account.to_account_info().key(),
            tip_amount: args.tip_amount,
        });

        Ok(())
    }
}