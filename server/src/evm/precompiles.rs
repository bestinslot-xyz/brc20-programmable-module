use db::DB;

use std::{str::FromStr, sync::Arc};

use revm::{
    precompile::{Error as PrecompileError, PrecompileResult},
    primitives::{Address, Bytes, PrecompileErrors, PrecompileOutput},
    ContextPrecompile, ContextStatefulPrecompile,
};

pub struct IdentityPrecompile;

impl ContextStatefulPrecompile<DB> for IdentityPrecompile {
    fn call(
        &self,
        bytes: &Bytes,
        gas_limit: u64,
        _evmctx: &mut revm::InnerEvmContext<DB>,
    ) -> PrecompileResult {
        identity_run(bytes, gas_limit)
    }
}

pub fn identity_run(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    println!("Running identity precompile");
    let gas_used = 100000;
    if gas_used > gas_limit {
        return Err(PrecompileErrors::Error(PrecompileError::OutOfGas));
    }
    Ok(PrecompileOutput {
        bytes: input.clone(),
        gas_used,
    })
}

pub fn load_precompiles() -> [(Address, ContextPrecompile<DB>); 2] {
    [
        (
            Address::from_str("0x00000000000000000000000000000000000000ff").unwrap(),
            ContextPrecompile::ContextStateful(Arc::new(IdentityPrecompile)),
        ),
        (
            Address::from_str("0x00000000000000000000000000000000000000fe").unwrap(),
            ContextPrecompile::ContextStateful(Arc::new(IdentityPrecompile)),
        ),
    ]
}
