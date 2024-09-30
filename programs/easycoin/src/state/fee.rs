use core::fmt;

use anchor_lang::{
    prelude::*,
    solana_program::sysvar::{self},
};

use crate::errors::*;
use crate::external_program::ComputeBudget;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum FeeIndex {
    SwapFeeNumerator = 0,
    SwapFeeDenominator,
    FeeIndexInvalid,
}

impl fmt::Display for FeeIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FeeIndex::SwapFeeNumerator => write!(f, "SwapFeeNumerator"),
            FeeIndex::SwapFeeDenominator => write!(f, "SwapFeeDenominator"),
            FeeIndex::FeeIndexInvalid => write!(f, "FeeIndexInvalid"),
        }
    }
}

#[account]
pub struct FeeRegistry {
    pub bump: u8,
    pub placeholder: Pubkey, // unused yet
    pub fees: Vec<u64>,
    pub fee_collectors: Vec<Pubkey>,
}

impl FeeRegistry {
    pub const FEE_INDEX_LEN: usize = FeeIndex::FeeIndexInvalid as usize;
    pub const MAX_FEE_COLLECTORS: usize = 16;

    pub fn size() -> usize {
        8 +  // discriminator
        1 +  // bump
        32 + // placeholder
        4  +  // length of fees
        8 * Self::FEE_INDEX_LEN + // fees
        4 +  // length of fee_collectors
        32 * Self::MAX_FEE_COLLECTORS // fee_collectors
    }

    pub fn get_fee_value(&self, index: FeeIndex) -> Result<u64> {

        require!(index != FeeIndex::FeeIndexInvalid, AgentError::FeeIndexInvalid);
        require!(self.fees.len() == Self::FEE_INDEX_LEN, AgentError::FeesLengthInvalid);

        Ok(self.fees[index as usize])
    }

    fn is_legal_fee_value(index: FeeIndex, value: u64) -> bool {
        match index {
            FeeIndex::SwapFeeNumerator => true,
            FeeIndex::SwapFeeDenominator => value != 0,
            _ => false,
        }
    }


    pub fn set_fee_value(&mut self, index: FeeIndex, value: u64) -> Result<()> {

        require!(index != FeeIndex::FeeIndexInvalid, AgentError::FeeIndexInvalid);
        require!(self.fees.len() == Self::FEE_INDEX_LEN, AgentError::FeesLengthInvalid);
        require!(Self::is_legal_fee_value(index, value), AgentError::InvalidFeeValue);

        self.fees[index as usize] = value;
        msg!("fee set: index={}, value={}", index, value);

        Ok(())
    }

    pub fn calculate_swap_fee(
        &self,
        wsol_balance_before: u64,
        wsol_balance_after: u64,
    ) -> Result<u64> {
        let swap_amount = (wsol_balance_after).abs_diff(wsol_balance_before);

        let swap_fee_numerator = self.get_fee_value(FeeIndex::SwapFeeNumerator)?;
        let swap_fee_denominator = self.get_fee_value(FeeIndex::SwapFeeDenominator)?;

        let swap_fee_u128 = u128::from(swap_amount)
            .checked_mul(swap_fee_numerator.into()) // in fact, it would never overflow
            .unwrap()
            .checked_div(swap_fee_denominator.into()) // in fact, swap_fee_denominator would never be 0
            .ok_or(AgentError::SwapFeeOverflow)?;

        require!(swap_fee_u128 <= u128::from(u64::MAX), AgentError::SwapFeeOverflow);

        return Ok(swap_fee_u128 as u64);
    }

    // 1st instruction => SetComputeUnitLimit
    // 2nd instruction => SetComputeUnitPrice
    // otherwise => error
    pub fn calculate_tx_fee(instructions_sysvar: &AccountInfo) -> Result<u64> {
        let instruction_set_compute_unit_limit =
            sysvar::instructions::load_instruction_at_checked(0, instructions_sysvar)?;
        require!(instruction_set_compute_unit_limit.program_id == ComputeBudget::id(), AgentError::ComputeBudgetNotAtFirst);

        let mut instruction_data_set_compute_unit_limit: [u8; 4] = [0; 4];
        // 1st byte is instruction discriminator, skip it
        instruction_data_set_compute_unit_limit
            .copy_from_slice(&instruction_set_compute_unit_limit.data[1..5]);
        let compute_unit_limit = u32::from_le_bytes(instruction_data_set_compute_unit_limit);

        let instruction_set_compute_unit_price =
            sysvar::instructions::load_instruction_at_checked(1, instructions_sysvar)?;
        require!(instruction_set_compute_unit_price.program_id == ComputeBudget::id(), AgentError::ComputeBudgetNotAtFirst);

        let mut instruction_data_set_compute_unit_price: [u8; 8] = [0; 8];
        // 1st byte is instruction discriminator, skip it
        instruction_data_set_compute_unit_price
            .copy_from_slice(&instruction_set_compute_unit_price.data[1..9]);
        let compute_unit_price = u64::from_le_bytes(instruction_data_set_compute_unit_price);

        #[cfg(feature = "enable-log")]
        msg!(
            "compute_unit_limit: {}  compute_unit_price: {}",
            compute_unit_limit,
            compute_unit_price
        );

        // only 1 signature
        const BASE_SIGNATURE_COST: u128 = 5000;
        // would never overflow, as compute_unit_limit is u32 and compute_unit_price is u64
        let tx_fee = BASE_SIGNATURE_COST
            + (u128::from(compute_unit_limit) * u128::from(compute_unit_price))
                .checked_add(1000000 - 1)
                .unwrap()
                .checked_div(1000000)
                .ok_or(AgentError::TxFeeOverflow)?;
        require!(tx_fee <= u128::from(u64::MAX), AgentError::TxFeeOverflow);

        Ok(tx_fee as u64)
    }

    pub fn add_fee_collector(&mut self, new_fee_collector: Pubkey) -> Result<()> {
        if self.fee_collectors.contains(&new_fee_collector) {
            msg!(
                "fee collector already registered before: {}",
                new_fee_collector.to_string()
            );
            return Ok(());
        }

        if self.fee_collectors.len() >= Self::MAX_FEE_COLLECTORS {
            msg!("fee collector limit reached: {}", Self::MAX_FEE_COLLECTORS);
            return err!(AgentError::FeeCollectorLimitReached);
        }

        self.fee_collectors.push(new_fee_collector);
        msg!("fee collector added: {}", new_fee_collector.to_string());
        Ok(())
    }

    pub fn remove_fee_collector(&mut self, fee_collector_to_remove: Pubkey) -> Result<()> {
        if !self.fee_collectors.contains(&fee_collector_to_remove) {
            msg!(
                "fee collector not registered before: {}",
                fee_collector_to_remove.to_string()
            );
            return Ok(());
        }

        self.fee_collectors
            .retain(|&x| x != fee_collector_to_remove);
        msg!(
            "fee collector removed: {}",
            fee_collector_to_remove.to_string()
        );
        Ok(())
    }

    pub fn is_fee_collector(&self, fee_collector: Pubkey) -> bool {
        self.fee_collectors.contains(&fee_collector)
    }
}
