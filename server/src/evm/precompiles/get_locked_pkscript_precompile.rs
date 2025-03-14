use db::DB;
use revm::{
    precompile::Error,
    primitives::{Bytes, PrecompileErrors, PrecompileOutput, PrecompileResult},
    ContextStatefulPrecompile, InnerEvmContext,
};

use solabi::{selector, FunctionEncoder, U256};

pub struct GetLockedPkScriptPrecompile;

const GET_LOCKED_PKSCRIPT: FunctionEncoder<(String, U256), (String,)> =
    FunctionEncoder::new(selector!("getLockedPkscript(string,uint256)"));

impl ContextStatefulPrecompile<DB> for GetLockedPkScriptPrecompile {
    fn call(
        &self,
        bytes: &Bytes,
        gas_limit: u64,
        _evmctx: &mut InnerEvmContext<DB>,
    ) -> PrecompileResult {
        let gas_used = 20000;
        let result = GET_LOCKED_PKSCRIPT.decode_params(&bytes);

        if result.is_err() {
            return Err(PrecompileErrors::Error(Error::Other(
                "Invalid params".to_string(),
            )));
        }

        let (pkscript, lock_block_count) = result.unwrap();
        let result = get_p2tr_lock_addr(&pkscript, lock_block_count.as_u64());

        if gas_used > gas_limit {
            return Err(PrecompileErrors::Error(Error::OutOfGas));
        }

        Ok(PrecompileOutput {
            bytes: Bytes::from(GET_LOCKED_PKSCRIPT.encode_returns(&(result,))),
            gas_used,
        })
    }
}

fn get_p2tr_lock_addr(pkscript: &String, lock_block_count: u64) -> String {
    "".to_string()
}