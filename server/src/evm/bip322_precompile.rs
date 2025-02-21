use db::DB;
use revm::{
    precompile::Error,
    primitives::{Bytes, PrecompileErrors, PrecompileOutput, PrecompileResult},
    ContextStatefulPrecompile,
};

use solabi::{selector, FunctionEncoder};

pub struct BIP322Precompile;

const VERIFY: FunctionEncoder<(solabi::Address, String, String), (bool,)> =
    FunctionEncoder::new(selector!("verify(address,string,string)"));

impl ContextStatefulPrecompile<DB> for BIP322Precompile {
    fn call(
        &self,
        bytes: &Bytes,
        gas_limit: u64,
        _evmctx: &mut revm::InnerEvmContext<DB>,
    ) -> PrecompileResult {
        let gas_used = 100000;
        let params = VERIFY.decode_params(&bytes).unwrap();
        println!("{:?}", params);

        // TODO: Implement the actual verification logic

        if gas_used > gas_limit {
            return Err(PrecompileErrors::Error(Error::OutOfGas));
        }

        Ok(PrecompileOutput {
            bytes: Bytes::from(VERIFY.encode_returns(&(true,))),
            gas_used,
        })
    }
}
