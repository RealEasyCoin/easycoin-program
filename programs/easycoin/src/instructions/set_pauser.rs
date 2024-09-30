use anchor_lang::prelude::*;

use crate::state::*;
use crate::errors::*;


#[derive(Accounts)]
pub struct SetPauser<'info> {
    #[account(
        mut,
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
        constraint = operator_registry.is_operator_registry_authority(operator_registry_authority.key()) @ AgentError::OperatorRegistryAuthorityNotAllowed
    )]
    pub operator_registry_authority: Signer<'info>,
}

impl SetPauser<'_> {
    pub fn set_pauser(ctx: Context<SetPauser>, pauser: Pubkey) -> Result<()> {

        require!(ctx.accounts.operator_registry.program_ok(), AgentError::ProgramStatusError);

        ctx.accounts.pause_registry.set_pauser(pauser)
    }
}