use anchor_lang::prelude::*;

use crate::errors::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UserAccountInfo {
    pub nonce: u32,
    pub due_fee: u64, // SOL
}

impl PartialEq for UserAccountInfo {
    fn eq(&self, other: &Self) -> bool {
        self.nonce == other.nonce
    }
}

#[account]
#[derive(Default)]
pub struct OwnerAccount {
    pub bump: u8,
    pub created_user_accounts: Vec<UserAccountInfo>, // record nonce is enough
}

impl OwnerAccount {
    const MAX_USER_ACCOUNTS: usize = 128;

    pub fn size() -> usize {
        8    + // anchor account discriminator
        1    + // bump
        4    + // created_user_accounts vector length
        (12 * Self::MAX_USER_ACCOUNTS) // created_user_accounts, up to MAX_USER_ACCOUNTS accounts
    }

    pub fn is_eglible_to_close(&self) -> bool {
        self.created_user_accounts.is_empty()
    }

    pub fn get_user_account_due_fee(&self, nonce: u32) -> Result<u64> {
        if let Some(user_account_info) =
            self.created_user_accounts.iter().find(|x| x.nonce == nonce)
        {
            Ok(user_account_info.due_fee)
        } else {
            #[cfg(feature = "enable-log")]
            msg!("User account with nonce {} does not exist", nonce);
            return err!(AgentError::UserAccountDoesNotExist);
        }
    }

    pub fn add_user_account_due_fee(&mut self, nonce: u32, due_fee_to_add: u64) -> Result<()> {
        if let Some(user_account_info) = self
            .created_user_accounts
            .iter_mut()
            .find(|x| x.nonce == nonce)
        {
            user_account_info.due_fee = user_account_info
                .due_fee
                .checked_add(due_fee_to_add)
                .ok_or(AgentError::DueFeeOverflow)?;
            #[cfg(feature = "enable-log")]
            msg!(
                "User account(nonce {}) due fee add {}, now is {}",
                nonce,
                due_fee_to_add,
                user_account_info.due_fee
            );
            return Ok(());
        } else {
            #[cfg(feature = "enable-log")]
            msg!("User account with nonce {} does not exist", nonce);
            return err!(AgentError::UserAccountDoesNotExist);
        }
    }

    pub fn sub_user_account_due_fee(&mut self, nonce: u32, due_fee_to_sub: u64) -> Result<()> {
        if let Some(user_account_info) = self
            .created_user_accounts
            .iter_mut()
            .find(|x| x.nonce == nonce)
        {
            user_account_info.due_fee = user_account_info
                .due_fee
                .checked_sub(due_fee_to_sub)
                .ok_or(AgentError::DueFeeOverflow)?;
            #[cfg(feature = "enable-log")]
            msg!(
                "User account(nonce {}) due fee sub {}, now is {}",
                nonce,
                due_fee_to_sub,
                user_account_info.due_fee
            );

            Ok(())
        } else {
            #[cfg(feature = "enable-log")]
            msg!("User account with nonce {} does not exist", nonce);
            return err!(AgentError::UserAccountDoesNotExist);
        }
    }

    pub fn add_user_account(&mut self, nonce: u32) -> Result<()> {
        let new_user_account_info = UserAccountInfo { nonce, due_fee: 0 };

        if self.created_user_accounts.contains(&new_user_account_info) {
            msg!("User account with nonce {} already exists", nonce);
            return err!(AgentError::UserAccountAlreadyExists);
        }

        if self.created_user_accounts.len() >= Self::MAX_USER_ACCOUNTS {
            msg!("User account limit reached: {}", Self::MAX_USER_ACCOUNTS);
            return err!(AgentError::UserAccountLimitReached);
        }

        self.created_user_accounts.push(new_user_account_info);
        msg!("User account (nonce {}) created", nonce);
        Ok(())
    }

    pub fn remove_user_account(&mut self, nonce: u32) -> Result<()> {
        let tmp_user_account_info = UserAccountInfo { nonce, due_fee: 0 };

        if !self.created_user_accounts.contains(&tmp_user_account_info) {
            msg!("User account with nonce {} does not exist", nonce);
            return err!(AgentError::UserAccountDoesNotExist);
        }

        self.created_user_accounts
            .retain(|x| x != &tmp_user_account_info);
        msg!("User account (nonce {}) closed", nonce);
        Ok(())
    }
}
