use anchor_lang::prelude::*;

use crate::errors::*;
use crate::state::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct SetFeeEntry {
    pub fee_index: FeeIndex,
    pub value: u64,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, PartialEq, Eq)]
pub enum ManageFeeOp {
    SetFee {
        fees_to_set: Vec<SetFeeEntry>,
    },
    AddFeeCollector {
        fee_collectors: Vec<Pubkey>,
    },
    RemoveFeeCollector {
        fee_collectors: Vec<Pubkey>,
    },
}

#[derive(Accounts)]
pub struct ManageFee<'info> {
    #[account(
        mut,
        seeds = [SEED_PREFIX, SEED_FEE],
        bump
    )]
    pub fees: Account<'info, FeeRegistry>,
    #[account(
        seeds = [SEED_PREFIX, SEED_OPERATOR],
        bump
    )]
    pub operator_registry: Account<'info, OperatorRegistry>,
    #[account(
        constraint = operator_registry.is_operator_registry_authority(operator_registry_authority.key()) @ AgentError::OperatorRegistryAuthorityNotAllowed
    )]
    pub operator_registry_authority: Signer<'info>,
}

impl ManageFee<'_> {
    pub fn manage_fee(ctx: Context<ManageFee>, op: ManageFeeOp) -> Result<()> {
        
        require!(ctx.accounts.operator_registry.program_ok(), AgentError::ProgramStatusError);

        let fees = &mut ctx.accounts.fees;

        match op {
            ManageFeeOp::SetFee { fees_to_set } => {
                for fee in fees_to_set.iter() {
                    fees.set_fee_value(fee.fee_index, fee.value)?;
                }
            }
            ManageFeeOp::AddFeeCollector { fee_collectors } => {
                for fee_collector in fee_collectors.iter() {
                    fees.add_fee_collector(*fee_collector)?;
                }
            }
            ManageFeeOp::RemoveFeeCollector { fee_collectors } => {
                for fee_collector in fee_collectors.iter() {
                    fees.remove_fee_collector(*fee_collector)?;
                }
            }
        }

        Ok(())
    }
}
