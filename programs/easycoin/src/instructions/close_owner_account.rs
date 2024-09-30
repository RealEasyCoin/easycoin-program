use anchor_lang::prelude::*;

use crate::errors::*;
use crate::state::*;

#[derive(Accounts)]
pub struct CloseOwnerAccount<'info> {
    #[account(
        mut,
        close = owner,
        constraint = owner_account.is_eglible_to_close() @ AgentError::OwnerAccountNotEligibleToClose,
        seeds = [SEED_PREFIX, SEED_OWNER, owner.key().as_ref()],
        bump
    )]
    pub owner_account: Account<'info, OwnerAccount>,
    pub owner: Signer<'info>, // The wallet owner, must sign the transaction
    #[account(
        seeds = [SEED_PREFIX, SEED_OPERATOR],
        bump
    )]
    pub operator_registry: Account<'info, OperatorRegistry>,
}

impl CloseOwnerAccount<'_> {
    pub fn close_owner_account(ctx: Context<CloseOwnerAccount>) -> Result<()> {
        require!(ctx.accounts.operator_registry.program_ok(), AgentError::ProgramStatusError);
        Ok(())
    }
}
