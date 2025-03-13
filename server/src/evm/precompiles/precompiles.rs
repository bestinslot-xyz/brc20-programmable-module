use db::DB;

use std::{str::FromStr, sync::Arc};

use revm::{primitives::Address, ContextPrecompile};

use super::{BIP322Precompile, BRC20Precompile, BTCPrecompile, LastSatLocationPrecompile};

pub fn load_precompiles() -> [(Address, ContextPrecompile<DB>); 4] {
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
        (
            Address::from_str("0x00000000000000000000000000000000000000fc").unwrap(),
            ContextPrecompile::ContextStateful(Arc::new(LastSatLocationPrecompile)),
        )
    ]
}
