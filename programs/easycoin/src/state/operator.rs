use anchor_lang::prelude::*;
use core::fmt;
use std::str::FromStr;

use crate::errors::*;

const DEFAULT_REGISTRY_AUTHORITY: &str = "zsWJ5xMC3A5JPJgSaaiHrfVastB8jqGZJjy4X8LTqex";

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Default)]
pub enum ProgramStatus {
    #[default]
    Uninitialized,
    Initialized,
    Paused,
}

impl fmt::Display for ProgramStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProgramStatus::Uninitialized => write!(f, "Uninitialized"),
            ProgramStatus::Initialized => write!(f, "Initialized"),
            ProgramStatus::Paused => write!(f, "Paused"),
        }
    }
}

#[account]
#[derive(Default)]
pub struct OperatorRegistry {
    pub bump: u8,
    pub program_status: ProgramStatus,
    pub operator_registry_authority: Pubkey,
    pub operators: Vec<Pubkey>,
}

impl OperatorRegistry {
    const MAX_OPERATORS: usize = 32;

    pub fn size() -> usize {
        8       + // anchor account discriminator
        1       + // bump
        1       + // program status
        32      + // operator_registry_authority
        4       + // operators vector length
        (32 * Self::MAX_OPERATORS) // operators, up to MAX_OPERATORS operators
    }

    pub fn default_registry_authority() -> Pubkey {
        Pubkey::from_str(DEFAULT_REGISTRY_AUTHORITY).unwrap()
    }

    pub fn is_operator_registry_authority(&self, authority: Pubkey) -> bool {
        self.operator_registry_authority == authority
    }

    pub fn transfer_registry_authority(&mut self, new_authority: Pubkey) -> Result<()> {
        let old_authority = self.operator_registry_authority;

        self.operator_registry_authority = new_authority;
        msg!(
            "operator registry authority transferred: {} => {}",
            old_authority,
            new_authority
        );

        Ok(())
    }

    pub fn add_operator(&mut self, new_operator: Pubkey) -> Result<()> {
        if self.operators.contains(&new_operator) {
            msg!("operator already registered before: {}", new_operator);
            return Ok(());
        }

        if self.operators.len() >= Self::MAX_OPERATORS {
            msg!("operator limit reached: {}", Self::MAX_OPERATORS);
            return err!(AgentError::OperatorLimitReached);
        }

        self.operators.push(new_operator);
        msg!("operator added: {}", new_operator);
        Ok(())
    }

    pub fn remove_operator(&mut self, operator_to_remove: Pubkey) -> Result<()> {
        if !self.operators.contains(&operator_to_remove) {
            msg!("operator not registered before: {}", operator_to_remove);
            return Ok(());
        }

        self.operators.retain(|x| x != &operator_to_remove);
        msg!("operator removed: {}", operator_to_remove);
        Ok(())
    }

    pub fn is_operator(&self, operator: Pubkey) -> bool {
        self.operators.contains(&operator)
    }

    pub fn pause(&mut self) -> Result<()> {
        if self.program_status == ProgramStatus::Initialized {
            self.program_status = ProgramStatus::Paused;
            msg!("program paused");
            Ok(())
        } else {
            msg!("program status error: {}", self.program_status);
            err!(AgentError::ProgramStatusError)
        }
    }

    pub fn unpause(&mut self) -> Result<()> {
        if self.program_status == ProgramStatus::Paused {
            self.program_status = ProgramStatus::Initialized;
            msg!("program unpaused");
            Ok(())
        } else {
            msg!("program status error: {}", self.program_status);
            err!(AgentError::ProgramStatusError)
        }
    }

    pub fn program_ok(&self) -> bool {
        self.program_status == ProgramStatus::Initialized
    }

}
