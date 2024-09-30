use anchor_lang::prelude::*;

pub use instructions::*;
pub use state::*;

pub mod errors;
pub mod instructions;
pub mod state;
pub mod events;
pub mod external_program;

declare_id!("easyTwKoYFtBTzmNqGYjKS5nZ9SvdTkhPxSHbBMnraY");

#[program]
pub mod easycoin {
    use super::*;

    /******* management related ********/

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Initialize::initialize(ctx)
    }

    pub fn manage(ctx: Context<Manage>, op: ManageOp) -> Result<()> {
        Manage::manage(ctx, op)
    }

    pub fn initialize_fee(ctx: Context<InitializeFee>, fee: InitializeFeeArgs) -> Result<()> {
        InitializeFee::initialize_fee(ctx, fee)
    }

    pub fn manage_fee(ctx: Context<ManageFee>, op: ManageFeeOp) -> Result<()> {
        ManageFee::manage_fee(ctx, op)
    }

    pub fn initialize_pause(ctx: Context<InitializePause>, pauser: Pubkey) -> Result<()> {
        InitializePause::initialize_pause(ctx, pauser)
    }

    pub fn set_pauser(ctx: Context<SetPauser>, pauser: Pubkey) -> Result<()> {
        SetPauser::set_pauser(ctx, pauser)
    }

    pub fn pause(ctx: Context<Pause>) -> Result<()> {
        Pause::pause(ctx)
    }

    pub fn unpause(ctx: Context<Pause>) -> Result<()> {
        Pause::unpause(ctx)
    }

    /******* user management related ********/

    pub fn create_owner_account(ctx: Context<CreateOwnerAccount>) -> Result<()> {
        CreateOwnerAccount::create_owner_account(ctx)
    }

    pub fn close_owner_account(ctx: Context<CloseOwnerAccount>) -> Result<()> {
        CloseOwnerAccount::close_owner_account(ctx)
    }

    /* user account */
    pub fn create_user_account(ctx: Context<CreateUserAccount>, args: CreateUserAccountArgs) -> Result<()> {
        CreateUserAccount::create_user_account(ctx, args)
    }

    pub fn withdraw(ctx: Context<Withdraw>, args: WithdrawArgs) -> Result<()> {
        Withdraw::withdraw(ctx, args)
    }

    /* user token account */
    pub fn create_user_token_account(ctx: Context<CreateUserTokenAccount>, args: CreateUserTokenAccountArgs) -> Result<()> {
        CreateUserTokenAccount::create_user_token_account(ctx, args)
    }
    
    pub fn close_user_token_account(ctx: Context<CloseUserTokenAccount>, args: CloseUserTokenAccountArgs) -> Result<()> {
        CloseUserTokenAccount::close_user_token_account(ctx, args)
    }

    pub fn transfer_and_sync_wsol(ctx: Context<TransferAndSyncWsol>, args: TransferAndSyncWsolArgs) -> Result<()> {
        TransferAndSyncWsol::transfer_and_sync_wsol(ctx, args)
    }

    pub fn swap_on_jupiter<'info>(ctx: Context<'_, '_, 'info, 'info, SwapOnJupiter>, args: SwapOnJupiterArgs) -> Result<()> {
        SwapOnJupiter::swap_on_jupiter(ctx, args)
    }

    pub fn swap_on_pumpfun<'info>(ctx: Context<'_, '_, 'info, 'info, SwapOnPumpfun>, args: SwapOnPumpfunArgs) -> Result<()> {
        SwapOnPumpfun::swap_on_pumpfun(ctx, args)
    }

    pub fn swap_on_pumpfun_v2<'info>(ctx: Context<'_, '_, 'info, 'info, SwapOnPumpfunV2>, args: SwapOnPumpfunV2Args) -> Result<()> {
        SwapOnPumpfunV2::swap_on_pumpfun_v2(ctx, args)
    }

    pub fn collect_fee(ctx: Context<CollectFee>, args: CollectFeeArgs) -> Result<()> {
        CollectFee::collect_fee(ctx, args)
    }

    pub fn tip_jito(ctx: Context<TipJito>, args: TipJitoArgs) -> Result<()> {
        TipJito::tip_jito(ctx, args)
    }

}




