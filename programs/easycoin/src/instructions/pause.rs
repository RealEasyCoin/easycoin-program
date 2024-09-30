use anchor_lang::prelude::*;

use crate::state::*;
use crate::errors::*;

#[derive(Accounts)]
pub struct Pause<'info> {
    #[account(
        constraint = pause_registry.is_pauser(pauser.key()) || operator_registry.is_operator_registry_authority(pauser.key())  @ AgentError::PauserNotAllowed
    )]
    pub pauser: Signer<'info>,
    #[account(
        seeds = [SEED_PREFIX, SEED_PAUSE],
        bump
    )]
    pub pause_registry: Account<'info, PauseRegistry>,
    #[account(
        mut,    
        seeds = [SEED_PREFIX, SEED_OPERATOR],
        bump
    )]
    pub operator_registry: Account<'info, OperatorRegistry>,
}

impl Pause<'_> {

    pub fn pause(ctx: Context<Pause>) -> Result<()> {
        ctx.accounts.operator_registry.pause()?;
        Ok(())
    }

    pub fn unpause(ctx: Context<Pause>) -> Result<()> {
        ctx.accounts.operator_registry.unpause()?;
        Ok(())
    }
}