use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke_signed},
};
use anchor_spl::token_interface::TokenAccount;

use crate::errors::*;
use crate::state::*;
use crate::external_program::Pumpfun;


#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct SwapOnPumpfunArgs {
    user_account_nonce: u32,
    pumpfun_data: Vec<u8>,
}


#[derive(Accounts)]
#[instruction(args: SwapOnPumpfunArgs)]
pub struct SwapOnPumpfun<'info> {
    #[account(
        mut,
        seeds = [SEED_PREFIX, SEED_USER, owner_account.key().as_ref(), &args.user_account_nonce.to_le_bytes()],
        bump
    )]
    pub user_account: SystemAccount<'info>,
    #[account(mut)]
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
    #[account(
        seeds = [SEED_PREFIX, SEED_FEE],
        bump
    )]
    pub fee_registry: Account<'info, FeeRegistry>,
    pub pumpfun_program: Program<'info, Pumpfun>,
}

impl<'info> SwapOnPumpfun<'info> {
    fn validate_user_token_account(
        user_account: SystemAccount,
        remaining_accounts: &'info [AccountInfo<'info>],
    ) -> Result<()> {
        // validate user account
        let user_account_info = remaining_accounts
            .get(6)
            .ok_or(AgentError::NotPumpfunTrade)?;
        require!(
            user_account_info.key() == user_account.key(),
            AgentError::PumpfunUserAccountInvalid
        );

        // validate user token account
        let user_token_account_info = remaining_accounts
            .get(5)
            .ok_or(AgentError::NotPumpfunTrade)?;
        let user_token_account: InterfaceAccount<'info, TokenAccount> =
            InterfaceAccount::try_from(user_token_account_info)?;
        require!(
            user_token_account.owner == user_account.key(),
            AgentError::PumpfunUserTokenAccountInvalid
        );

        Ok(())
    }

    pub fn swap_on_pumpfun(
        ctx: Context<'_, '_, 'info, 'info, SwapOnPumpfun>,
        args: SwapOnPumpfunArgs,
    ) -> Result<()> {
        require!(
            ctx.accounts.operator_registry.program_ok(),
            AgentError::ProgramStatusError
        );

        let remaining_accounts = ctx.remaining_accounts;
        let data = args.pumpfun_data;
        let user_account = &ctx.accounts.user_account;

        Self::validate_user_token_account(user_account.clone(), remaining_accounts)?;

        #[cfg(feature = "enable-log")]
        msg!("user token account is valid");
        // get user account lamports before
        let user_account_lamports_before = user_account.lamports();
        #[cfg(feature = "enable-log")]
        msg!(
            "user_account_lamports before: {}",
            user_account_lamports_before
        );

        let mut accounts: Vec<AccountMeta> = remaining_accounts
            .iter()
            .map(|acc| AccountMeta {
                pubkey: *acc.key,
                is_signer: acc.is_signer,
                is_writable: acc.is_writable,
            })
            .collect();

        if let Some(signer_account) = accounts.get_mut(6) {
            signer_account.is_signer = true;
        }

        let accounts_infos: Vec<AccountInfo> = remaining_accounts
            .iter()
            .map(|acc| AccountInfo { ..acc.clone() })
            .collect();

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

        invoke_signed(
            &Instruction {
                program_id: ctx.accounts.pumpfun_program.key(),
                accounts,
                data,
            },
            &accounts_infos,
            signer_seeds,
        )?;

        // ? Do we need to reload the user account here?
        let user_account_lamports_after = user_account.lamports();
        let trade_fee = ctx
            .accounts
            .fee_registry
            .calculate_swap_fee(user_account_lamports_before, user_account_lamports_after)?;

        // record swap fee
        ctx.accounts
        .owner_account
        .add_user_account_due_fee(args.user_account_nonce, trade_fee)?;

        let required_rent: u64 = Rent::get()?.minimum_balance(user_account.data_len());
        let user_account_balance = user_account.lamports() - required_rent;
        
        let due_fee = ctx.accounts.owner_account.get_user_account_due_fee(args.user_account_nonce)?;
        require!(
            user_account_balance >= due_fee,
            AgentError::UserAccountBalanceNotEnough
        );

        Ok(())
    }

}
