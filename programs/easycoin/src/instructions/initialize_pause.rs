use anchor_lang::prelude::*;

use crate::errors::*;
use crate::state::*;

#[derive(Accounts)]
pub struct InitializePause<'info> {
    #[account(
        init,
        payer = operator_registry_authority,
        space = PauseRegistry::size(),
        seeds = [SEED_PREFIX, SEED_PAUSE],
        bump
    )]
    pub pause_registry: Account<'info, PauseRegistry>,
    #[account(
        seeds = [SEED_PREFIX, SEED_OPERATOR],
        bump
    )]
    pub operator_registry: Account<'info, OperatorRegistry>,
    #[account(
        mut,
        constraint = operator_registry.is_operator_registry_authority(operator_registry_authority.key()) @ AgentError::OperatorRegistryAuthorityNotAllowed
    )]
    pub operator_registry_authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl InitializePause<'_> {
    pub fn initialize_pause(ctx: Context<InitializePause>, pauser: Pubkey) -> Result<()> {
        let pause_registry = &mut ctx.accounts.pause_registry;

        pause_registry.bump = ctx.bumps.pause_registry;
        pause_registry.pauser = pauser;

        Ok(())
    }
}
