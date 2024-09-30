use anchor_lang::prelude::*;

#[event]
pub struct UserTokenAccountClosedEvent {
    pub token_account: Pubkey,
    pub user_account: Pubkey,
    pub lamports: u64,
}

#[event]
pub struct FeeCollectedEvent {
    pub user_account: Pubkey,
    pub tx_fee: u64,
    pub trade_fee: u64,
}

#[event]
pub struct TipJitoEvent {
    pub user_account: Pubkey,
    pub tip_amount: u64,
}