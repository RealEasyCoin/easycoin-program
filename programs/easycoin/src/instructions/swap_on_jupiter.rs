use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke_signed},
};
use anchor_spl::token_interface::TokenAccount;

use crate::errors::*;
use crate::external_program::Jupiter;
use crate::state::*;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum JupiterRouteType {
    Route,
    SharedAccountRoute,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct SwapOnJupiterArgs {
    user_account_nonce: u32,
    jupiter_data: Vec<u8>,
}

#[derive(Accounts)]
#[instruction(args: SwapOnJupiterArgs)]
pub struct SwapOnJupiter<'info> {
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
    pub jupiter_program: Program<'info, Jupiter>,
}

impl<'info> SwapOnJupiter<'info> {
    pub fn route_type(data: &[u8]) -> Result<JupiterRouteType> {
        const ROUTE_DISCRIMINATOR: &[u8] = &[229, 23, 203, 151, 122, 227, 173, 42];
        const SHARED_ACCOUNT_ROUTE_DISCRIMINATOR: &[u8] = &[193, 32, 155, 51, 65, 214, 156, 129];

        require!(data.len() >= 8, AgentError::NotSwapOnJupiter);

        match &data[..8] {
            ROUTE_DISCRIMINATOR => Ok(JupiterRouteType::Route),
            SHARED_ACCOUNT_ROUTE_DISCRIMINATOR => Ok(JupiterRouteType::SharedAccountRoute),
            _ => err!(AgentError::NotSwapOnJupiter),
        }
    }

    fn validate_destination_token_account(
        user_account: SystemAccount,
        route_type: JupiterRouteType,
        remaining_accounts: &'info [AccountInfo<'info>],
    ) -> Result<InterfaceAccount<'info, TokenAccount>> {
        match route_type {
            JupiterRouteType::Route => {
                let user_destination_token_account_info = remaining_accounts
                    .get(3)
                    .ok_or(AgentError::NotJupiterRoute)?;
                #[cfg(feature = "enable-log")]
                msg!(
                    "user_destination_token_account_info: {}",
                    user_destination_token_account_info.key().to_string()
                );

                let user_destination_token_account: InterfaceAccount<'info, TokenAccount> =
                    InterfaceAccount::try_from(user_destination_token_account_info)?;
                require!(
                    user_destination_token_account.owner == user_account.key(),
                    AgentError::JupiterRouteDestinationInvalid
                );

                #[cfg(feature = "enable-log")]
                msg!("destination token account authority is valid");

                let destination_token_account_info = remaining_accounts
                    .get(4)
                    .ok_or(AgentError::NotJupiterRoute)?;
                // if equal to Jupiter::id(), destination_token_account is user_destination_token_account
                // require!(
                //     destination_token_account_info.key() == Jupiter::id()
                //         || destination_token_account_info.key()
                //             == user_destination_token_account.key(),
                //     AgentError::JupiterRouteDestinationInvalid
                // );

                require!(
                    destination_token_account_info.key() == user_destination_token_account.key(),
                    AgentError::JupiterRouteDestinationInvalid
                );
                Ok(user_destination_token_account)
            }
            JupiterRouteType::SharedAccountRoute => {
                return err!(AgentError::NotJupiterRoute);
            }
        }
    }

    fn validate_source_token_account(
        user_account: SystemAccount,
        route_type: JupiterRouteType,
        remaining_accounts: &'info [AccountInfo<'info>],
    ) -> Result<InterfaceAccount<'info, TokenAccount>> {
        // verify user_account matches the info in remaining accounts
        match route_type {
            JupiterRouteType::Route => {
                // verify user transfer authority matches
                let user_transfer_authority_account_info = remaining_accounts
                    .get(1)
                    .ok_or(AgentError::NotJupiterRoute)?;
                require!(
                    user_transfer_authority_account_info.key() == user_account.key(),
                    AgentError::JupiterRouteSourceInvalid
                );

                // verify user source token account matches
                let user_source_token_account_info = remaining_accounts
                    .get(2)
                    .ok_or(AgentError::NotJupiterRoute)?;
                let user_source_token_account: InterfaceAccount<'info, TokenAccount> =
                    InterfaceAccount::try_from(user_source_token_account_info)?;
                require!(
                    user_source_token_account.owner == user_account.key(),
                    AgentError::JupiterRouteSourceInvalid
                );

                Ok(user_source_token_account)
            }
            JupiterRouteType::SharedAccountRoute => {
                return err!(AgentError::NotJupiterRoute);
            }
        }
    }

    pub fn swap_on_jupiter(
        ctx: Context<'_, '_, 'info, 'info, SwapOnJupiter>,
        args: SwapOnJupiterArgs,
    ) -> Result<()> {
        require!(
            ctx.accounts.operator_registry.program_ok(),
            AgentError::ProgramStatusError
        );

        let remaining_accounts = ctx.remaining_accounts;
        let data = args.jupiter_data;
        let route_type = Self::route_type(&data)?;
        let user_account = &ctx.accounts.user_account;

        let user_source_token_account = Self::validate_source_token_account(
            user_account.clone(),
            route_type,
            remaining_accounts,
        )?;

        #[cfg(feature = "enable-log")]
        msg!("source token account is valid");

        let user_destination_token_account = Self::validate_destination_token_account(
            user_account.clone(),
            route_type,
            remaining_accounts,
        )?;

        #[cfg(feature = "enable-log")]
        msg!("destination token account is valid");

        #[cfg(feature = "enable-log")]
        msg!("account validation passed!");

        let mut wsol_token_account = match (
            user_source_token_account.is_native(),
            user_destination_token_account.is_native(),
        ) {
            (true, false) => user_source_token_account,
            (false, true) => user_destination_token_account,
            _ => return err!(AgentError::NoWsolTokenAccount),
        };

        let wsol_balance_before = wsol_token_account.amount;

        let mut accounts: Vec<AccountMeta> = remaining_accounts
            .iter()
            .map(|acc| AccountMeta {
                pubkey: *acc.key,
                is_signer: acc.is_signer,
                is_writable: acc.is_writable,
            })
            .collect();

        // specify user account (2nd account in remaining_accounts) as signer
        if let Some(second_account) = accounts.get_mut(1) {
            second_account.is_signer = true;
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
                program_id: ctx.accounts.jupiter_program.key(),
                accounts,
                data,
            },
            &accounts_infos,
            signer_seeds,
        )?;

        wsol_token_account.reload()?;
        let wsol_balance_after = wsol_token_account.amount;
        let trade_fee = ctx
            .accounts
            .fee_registry
            .calculate_swap_fee(wsol_balance_before, wsol_balance_after)?;

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
