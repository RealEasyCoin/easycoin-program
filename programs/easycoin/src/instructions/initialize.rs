use anchor_lang::prelude::*;

use crate::errors::*;
use crate::state::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = OperatorRegistry::size(),
        seeds = [SEED_PREFIX, SEED_OPERATOR],
        bump
    )]
    pub operator_registry: Account<'info, OperatorRegistry>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl Initialize<'_> {
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let operator_registry = &mut ctx.accounts.operator_registry;

        if operator_registry.program_status != ProgramStatus::Uninitialized {
            return err!(AgentError::ProgramStatusError);
        }

        operator_registry.bump = ctx.bumps.operator_registry;
        operator_registry.operators = vec![];
        operator_registry.operator_registry_authority =
            OperatorRegistry::default_registry_authority();

        msg!(
            "default operator_registry_authority: {}",
            operator_registry.operator_registry_authority.to_string()
        );

        operator_registry.program_status = ProgramStatus::Initialized;

        Ok(())
    }
}
