use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct PauseRegistry {
    pub bump: u8,
    pub pauser: Pubkey,
}

impl PauseRegistry {
    pub fn size() -> usize {
        8 + // anchor account discriminator
        1 + // bump
        32 // pauser
    }

    pub fn set_pauser(&mut self, pauser: Pubkey) -> Result<()> {
        let old_pauser = self.pauser;
        self.pauser = pauser;
        msg!("pauser set: {} => {}", old_pauser, pauser);
        Ok(())
    }

    pub fn is_pauser(&self, pauser: Pubkey) -> bool {
        self.pauser == pauser
    }
}
