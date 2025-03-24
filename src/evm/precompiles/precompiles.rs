use std::str::FromStr;
use std::sync::Arc;

use revm::primitives::Address;
use revm::ContextPrecompile;

use crate::db::DB;
use crate::evm::precompiles::{
    BIP322Precompile, BRC20Precompile, BTCTxDetailsPrecompile, GetLockedPkScriptPrecompile,
    LastSatLocationPrecompile,
};

pub fn load_precompiles() -> [(Address, ContextPrecompile<DB>); 5] {
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
            ContextPrecompile::ContextStateful(Arc::new(BTCTxDetailsPrecompile)),
        ),
        (
            Address::from_str("0x00000000000000000000000000000000000000fc").unwrap(),
            ContextPrecompile::ContextStateful(Arc::new(LastSatLocationPrecompile)),
        ),
        (
            Address::from_str("0x00000000000000000000000000000000000000fb").unwrap(),
            ContextPrecompile::ContextStateful(Arc::new(GetLockedPkScriptPrecompile)),
        ),
    ]
}
