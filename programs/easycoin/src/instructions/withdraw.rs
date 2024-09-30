use anchor_lang::prelude::*;
use anchor_lang::system_program;

use crate::errors::*;
use crate::state::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum WithdrawOp {
    Withdraw { amount: u64 },
    WithdrawAll,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct WithdrawArgs {
    pub user_account_nonce: u32, // nonce for account creation
    pub withdraw_op: WithdrawOp,
}

#[derive(Accounts)]
#[instruction(args: WithdrawArgs)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [SEED_PREFIX, SEED_USER, owner_account.key().as_ref(), args.user_account_nonce.to_le_bytes().as_ref()],
        bump
    )]
    pub user_account: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [SEED_PREFIX, SEED_OWNER, owner.key().as_ref()],
        bump
    )]
    pub owner_account: Account<'info, OwnerAccount>,
    pub owner: Signer<'info>, // The account owner, must sign the transaction
    pub system_program: Program<'info, System>,
    #[account(
        seeds = [SEED_PREFIX, SEED_OPERATOR],
        bump
    )]
    pub operator_registry: Account<'info, OperatorRegistry>,
}

impl Withdraw<'_> {
    pub fn withdraw(ctx: Context<Withdraw>, args: WithdrawArgs) -> Result<()> {

        require!(ctx.accounts.operator_registry.program_ok(), AgentError::ProgramStatusError);

        let owner_account = &mut ctx.accounts.owner_account;
        let user_account = &mut ctx.accounts.user_account;
        let owner = &mut ctx.accounts.owner;

        let due_fee = owner_account.get_user_account_due_fee(args.user_account_nonce)?;
        if due_fee > 0 {
            // in case there is due fee for user account
            // if this branch is reached, means operator forgot to collect trade fee
            msg!("Due fee not paid for user account (nonce {}): {}", args.user_account_nonce, due_fee);
            return err!(AgentError::DueFeeNotPaid);
        }

        let user_account_lamports = user_account.lamports();
        #[cfg(feature = "enable-log")]
        msg!("user_account_lamports: {}", user_account_lamports);

        let withdraw_amount = match args.withdraw_op {
            WithdrawOp::Withdraw { amount } => {
                // the account would not be closed
                let required_rent = Rent::get()?.minimum_balance(user_account.data_len());

                let user_account_balance = user_account_lamports
                    .checked_sub(required_rent)
                    .ok_or(AgentError::UserAccountBalanceNotEnough)?;

                if user_account_balance < amount {
                    msg!(
                        "user account balance is {}, while amount is {}",
                        user_account_balance,
                        amount
                    );
                    return err!(AgentError::UserAccountBalanceNotEnough);
                }
                amount
            }
            // close user account
            WithdrawOp::WithdrawAll => user_account_lamports, 
        };

        // transfer SOL to owner
        let owner_account_key = owner_account.key();
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
                    from: user_account.to_account_info(),
                    to: owner.to_account_info(),
                },
                signer_seeds,
            ),
            withdraw_amount,
        )?;

        if args.withdraw_op == WithdrawOp::WithdrawAll {
            // remove user account info from owner account
            owner_account.remove_user_account(args.user_account_nonce)?;
        }

        Ok(())
    }
}
