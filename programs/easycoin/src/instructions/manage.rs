use anchor_lang::prelude::*;

use crate::errors::*;
use crate::state::*;

#[derive(AnchorDeserialize, AnchorSerialize, Clone, PartialEq, Eq)]
pub enum ManageOp {
    TransferAuthority {
        new_authority: Pubkey,
    },
    AddOperator {
        operators: Vec<Pubkey>,
    },
    RemoveOperator {
        operators: Vec<Pubkey>,
    },
}

#[derive(Accounts)]
pub struct Manage<'info> {
    #[account(
        mut,
        seeds = [SEED_PREFIX, SEED_OPERATOR],
        bump
    )]
    pub operator_registry: Account<'info, OperatorRegistry>,
    #[account(
        constraint = operator_registry.is_operator_registry_authority(operator_registry_authority.key()) @ AgentError::OperatorRegistryAuthorityNotAllowed
    )]
    pub operator_registry_authority: Signer<'info>,
}

impl Manage<'_> {
    pub fn manage(ctx: Context<Manage>, op: ManageOp) -> Result<()> {

        require!(ctx.accounts.operator_registry.program_ok(), AgentError::ProgramStatusError);

        let operator_registry = &mut ctx.accounts.operator_registry;

        match op {
            ManageOp::TransferAuthority { new_authority } => {
                operator_registry.transfer_registry_authority(new_authority)?;
            }
            ManageOp::AddOperator { operators } => {
                for operator in operators.iter() {
                    operator_registry.add_operator(*operator)?;
                }
            }
            ManageOp::RemoveOperator { operators } => {
                for operator in operators.iter() {
                    operator_registry.remove_operator(*operator)?;
                }
            }
        }

        Ok(())
    }
}
