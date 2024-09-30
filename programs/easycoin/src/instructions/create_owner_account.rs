use anchor_lang::prelude::*;

use crate::state::*;
use crate::errors::*;

/* Create Owner Account */
#[derive(Accounts)]
pub struct CreateOwnerAccount<'info> {
    #[account(
        init, 
        payer = owner, 
        space = OwnerAccount::size(),
        seeds = [SEED_PREFIX, SEED_OWNER, owner.key().as_ref()],
        bump
    )]
    pub owner_account: Account<'info, OwnerAccount>,

    #[account(mut)]            
    pub owner: Signer<'info>,  // The wallet owner, must sign the transaction
    pub system_program: Program<'info, System>,
    #[account(
        seeds = [SEED_PREFIX, SEED_OPERATOR],
        bump
    )]
    pub operator_registry: Account<'info, OperatorRegistry>,
}


impl CreateOwnerAccount<'_> {

    pub fn create_owner_account(ctx: Context<Self>) -> Result<()> {

        require!(ctx.accounts.operator_registry.program_ok(), AgentError::ProgramStatusError);

        let owner_account = &mut ctx.accounts.owner_account;
        owner_account.bump = ctx.bumps.owner_account;
        owner_account.created_user_accounts = vec![];
        
        Ok(())
    }

}