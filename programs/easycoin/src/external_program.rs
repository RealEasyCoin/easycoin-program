use anchor_lang::prelude::*;

mod pumpfun {
    use anchor_lang::declare_id;

    declare_id!("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");
}


#[derive(Clone)]
pub struct Pumpfun;

impl anchor_lang::Id for Pumpfun {
    fn id() -> Pubkey {
        pumpfun::id()
    }
}



mod jupiter {
    use anchor_lang::declare_id;

    declare_id!("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4");
}

#[derive(Clone)]
pub struct Jupiter;

impl anchor_lang::Id for Jupiter {
    fn id() -> Pubkey {
        jupiter::id()
    }
}


mod jito_tip_program {
    use anchor_lang::declare_id;

    declare_id!("T1pyyaTNZsKv2WcRAB8oVnk93mLJw2XzjtVYqCsaHqt");
}

#[derive(Clone)]
pub struct JitoTipProgram;

impl anchor_lang::Id for JitoTipProgram {
    fn id() -> Pubkey {
        jito_tip_program::id()
    }
}


mod compute_budget {
    use anchor_lang::declare_id;

    declare_id!("ComputeBudget111111111111111111111111111111");
}

#[derive(Clone)]
pub struct ComputeBudget;

impl anchor_lang::Id for ComputeBudget {
    fn id() -> Pubkey {
        compute_budget::id()
    }
}