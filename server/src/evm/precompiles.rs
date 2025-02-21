use db::DB;

use std::{str::FromStr, sync::Arc};

use revm::{
    primitives::Address,
    ContextPrecompile,
};

use super::{BIP322Precompile, BRC20Precompile, BTCPrecompile};

pub fn load_precompiles() -> [(Address, ContextPrecompile<DB>); 3] {
    [
        (
            Address::from_str("0x00000000000000000000000000000000000000ff").unwrap(),
            ContextPrecompile::ContextStateful(Arc::new(BRC20Precompile)),
        ),
        (
            Address::from_str("0x00000000000000000000000000000000000000fe").unwrap(),
            ContextPrecompile::ContextStateful(Arc::new(BIP322Precompile)),
        ),
        (
            Address::from_str("0x00000000000000000000000000000000000000fd").unwrap(),
            ContextPrecompile::ContextStateful(Arc::new(BTCPrecompile)),
        ),
    ]
}
