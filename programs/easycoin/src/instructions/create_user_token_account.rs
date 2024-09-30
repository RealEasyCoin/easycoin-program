use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{self, AssociatedToken, Create},
    token_interface::{Mint, TokenInterface},
};

use crate::errors::*;
use crate::state::*;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct CreateUserTokenAccountArgs {
    pub user_account_nonce: u32,
}

#[derive(Accounts)]
#[instruction(args: CreateUserTokenAccountArgs)]
pub struct CreateUserTokenAccount<'info> {
    #[account(mut)]
    /// CHECK: token account to create (via CPI)
    pub token_account: UncheckedAccount<'info>,
    #[account(
        mint::token_program = token_program,
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        seeds = [SEED_PREFIX, SEED_USER, owner_account.key().as_ref(), &args.user_account_nonce.to_le_bytes()],
        bump
    )]
    pub user_account: SystemAccount<'info>,
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
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl CreateUserTokenAccount<'_> {
    pub fn create_user_token_account(
        ctx: Context<CreateUserTokenAccount>,
        args: CreateUserTokenAccountArgs,
    ) -> Result<()> {

        require!(ctx.accounts.operator_registry.program_ok(), AgentError::ProgramStatusError);

        let user_account = &mut ctx.accounts.user_account;

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
        msg!(
            "create_user_token_account {} for user account (nonce {})",
            ctx.accounts.token_account.key().to_string(), args.user_account_nonce
        );
        associated_token::create_idempotent(CpiContext::new_with_signer(
            ctx.accounts.associated_token_program.to_account_info(),
            Create {
                payer: user_account.to_account_info(),
                associated_token: ctx.accounts.token_account.to_account_info(),
                authority: user_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            },
            signer_seeds,
        ))
    }
}
