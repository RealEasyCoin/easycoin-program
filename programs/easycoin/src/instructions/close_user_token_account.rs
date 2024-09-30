use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, TokenAccount, TokenInterface};

use crate::errors::*;
use crate::events::*;
use crate::state::*;

macro_rules! try_from {
    ($ty: ty, $acc: expr) => {
        <$ty>::try_from(unsafe { core::mem::transmute::<_, &AccountInfo<'_>>($acc.as_ref()) })
    };
}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct CloseUserTokenAccountArgs {
    pub user_account_nonce: u32,
}

#[derive(Accounts)]
#[instruction(args: CloseUserTokenAccountArgs)]
pub struct CloseUserTokenAccount<'info> {
    #[account(mut)]
    /// CHECK: token account to close, it may not exist
    pub token_account: UncheckedAccount<'info>,
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
    pub system_program: Program<'info, System>,
}

impl CloseUserTokenAccount<'_> {
    pub fn close_user_token_account(
        ctx: Context<CloseUserTokenAccount>,
        args: CloseUserTokenAccountArgs,
    ) -> Result<()> {

        require!(ctx.accounts.operator_registry.program_ok(), AgentError::ProgramStatusError);

        let user_token_account_lamports = ctx.accounts.token_account.lamports();
        // if the user token account does not exist, return
        if user_token_account_lamports == 0 {
            #[cfg(feature = "enable-log")]
            msg!("User token account does not exist");
            return Ok(());
        }

        let token_account = try_from!(InterfaceAccount<TokenAccount>, ctx.accounts.token_account)?;
        if !token_account.is_native() && token_account.amount != 0 {
            // token account has balance, return OK rather than error
            #[cfg(feature = "enable-log")]
            msg!("User token account has balance, not close");
            return Ok(());
        }

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

        token_interface::close_account(CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token_interface::CloseAccount {
                account: ctx.accounts.token_account.to_account_info(),
                destination: ctx.accounts.user_account.to_account_info(),
                authority: ctx.accounts.user_account.to_account_info(), // rent would be returned to user account
            },
            signer_seeds,
        ))?;

        emit!(UserTokenAccountClosedEvent {
            token_account: ctx.accounts.token_account.key(),
            user_account: ctx.accounts.user_account.key(),
            lamports: user_token_account_lamports,
        });
        Ok(())
    }
}
