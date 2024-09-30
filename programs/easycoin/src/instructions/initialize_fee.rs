use anchor_lang::prelude::*;

use crate::state::*;
use crate::errors::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub struct InitializeFeeArgs {
    pub swap_fee_numerator: u64,
    pub swap_fee_denominator: u64,
}


#[derive(Accounts)]
pub struct InitializeFee<'info> {
    #[account(
        init,
        payer = operator_registry_authority,
        space = FeeRegistry::size(),
        seeds = [SEED_PREFIX, SEED_FEE],
        bump
    )]
    pub fee_registry: Account<'info, FeeRegistry>,
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

impl InitializeFee<'_> {
    pub fn initialize_fee(ctx: Context<InitializeFee>, args: InitializeFeeArgs) -> Result<()> {
        let fee_registry = &mut ctx.accounts.fee_registry;

        fee_registry.bump = ctx.bumps.fee_registry;
        fee_registry.fees = vec![0; FeeRegistry::FEE_INDEX_LEN];
        fee_registry.fee_collectors = vec![];

        fee_registry.set_fee_value(FeeIndex::SwapFeeNumerator, args.swap_fee_numerator)?;
        fee_registry.set_fee_value(FeeIndex::SwapFeeDenominator, args.swap_fee_denominator)?;

        Ok(())
    }
}