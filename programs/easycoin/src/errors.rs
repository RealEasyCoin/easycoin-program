
use anchor_lang::prelude::*;

#[error_code]
pub enum AgentError {
    // operator registry related
    ProgramStatusError,
    OperatorRegistryNotUninitialized,
    OperatorRegistryAuthorityIsSame,
    FeeCollectorLimitReached,
    OperatorLimitReached,

    OperatorRegistryAuthorityNotAllowed,
    OperatorNotAllowed,

    PauserNotAllowed,
    PauserLimitReached,
    PauseRegistryAuthorityNotAllowed,

    JitoTipAccountInvalid,

    UserAccountAlreadyExists,
    UserAccountDoesNotExist,
    UserAccountBalanceNotEnough,
    UserAccountBalanceMayNotEnough,
    UserAccountLimitReached,

    OwnerAccountNotEligibleToClose,

    NotSwapOnJupiter,
    NotJupiterRoute,
    JupiterRouteSourceInvalid,
    JupiterRouteDestinationInvalid,
    NoWsolTokenAccount,

    NotPumpfunTrade,
    PumpfunUserAccountInvalid,
    PumpfunUserTokenAccountInvalid,

    DueFeeNotPaid,
    SwapFeeDenominatorZero,
    InvalidFeeValue,
    TxFeeOverflow,

    SwapAmountOverflow,
    PumpfunExceedSlippage,

    FeeCollectorInvalid,
    ComputeBudgetNotAtFirst,
    DueFeeOverflow, 
    SwapFeeOverflow,
    // TestError,
    FeeIndexInvalid,
    FeesLengthInvalid,
}
