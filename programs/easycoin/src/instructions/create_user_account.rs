use anchor_lang::prelude::*;

use crate::state::*;
use crate::errors::*;

/* Create User Account */
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateUserAccountArgs {
    pub nonce: u32,  // nonce for account creation
}

/* Create User Account */
#[derive(Accounts)]
#[instruction(args: CreateUserAccountArgs)]
pub struct CreateUserAccount<'info> {
    #[account(
        init, 
        payer = owner,
        owner = system_program.key(),
        space = 0,
        seeds = [SEED_PREFIX, SEED_USER, owner_account.key().as_ref(), &args.nonce.to_le_bytes()],
        bump 
    )]
    /// CHECK: create user account and assign its owner to system program
    pub user_account: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [SEED_PREFIX, SEED_OWNER, owner.key().as_ref()],
        bump
    )]
    pub owner_account: Account<'info, OwnerAccount>,
    #[account(mut)]
    pub owner: Signer<'info>,  // The account owner, must sign the transaction
    pub system_program: Program<'info, System>,
    #[account(
        seeds = [SEED_PREFIX, SEED_OPERATOR],
        bump
    )]
    pub operator_registry: Account<'info, OperatorRegistry>,
}

impl CreateUserAccount<'_> {

    pub fn create_user_account(ctx: Context<CreateUserAccount>, args: CreateUserAccountArgs) -> Result<()> {

        require!(ctx.accounts.operator_registry.program_ok(), AgentError::ProgramStatusError);

        /* record info to owner account */
        ctx.accounts.owner_account.add_user_account(args.nonce)
    }   
}