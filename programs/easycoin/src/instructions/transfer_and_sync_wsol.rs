use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::{self, spl_token, SyncNative, Token, TokenAccount};

use crate::errors::*;
use crate::state::*;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct TransferAndSyncWsolArgs {
    user_account_nonce: u32,
    amount: u64,
}

#[derive(Accounts)]
#[instruction(args: TransferAndSyncWsolArgs)]
pub struct TransferAndSyncWsol<'info> {
    #[account(
        mut,
        token::mint = spl_token::native_mint::id(),
        token::authority = user_account,
    )]
    pub wsol_token_account: Account<'info, TokenAccount>,

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
    pub token_program: Program<'info, Token>, // must be token program, not token-2022 program
    pub system_program: Program<'info, System>,
}

impl TransferAndSyncWsol<'_> {
    pub fn transfer_and_sync_wsol(
        ctx: Context<TransferAndSyncWsol>,
        args: TransferAndSyncWsolArgs,
    ) -> Result<()> {

        require!(ctx.accounts.operator_registry.program_ok(), AgentError::ProgramStatusError);

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

        system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.user_account.to_account_info(),
                    to: ctx.accounts.wsol_token_account.to_account_info(),
                },
                signer_seeds,
            ),
            args.amount,
        )?;
        #[cfg(feature = "enable-log")]
        msg!(
            "transfer sol to wsol token account complete! amount: {}",
            args.amount
        );

        token::sync_native(CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            SyncNative {
                account: ctx.accounts.wsol_token_account.to_account_info(),
            },
        ))?;

        Ok(())
    }
}
