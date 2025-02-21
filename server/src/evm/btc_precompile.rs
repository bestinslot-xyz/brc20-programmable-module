use db::DB;
use revm::{
    precompile::Error,
    primitives::{Bytes, PrecompileErrors, PrecompileOutput, PrecompileResult},
    ContextStatefulPrecompile,
};
use solabi::{selector, FunctionEncoder, U256};

pub struct BTCPrecompile;

const TX_DETAILS: FunctionEncoder<String, (String, U256, U256)> =
    FunctionEncoder::new(selector!("getTxDetails(string)"));

impl ContextStatefulPrecompile<DB> for BTCPrecompile {
    fn call(
        &self,
        bytes: &Bytes,
        gas_limit: u64,
        _evmctx: &mut revm::InnerEvmContext<DB>,
    ) -> PrecompileResult {
        let params = TX_DETAILS.decode_params(&bytes).unwrap();
        println!("{:?}", params);

        // TODO: Implement the actual logic

        let gas_used = 100000;
        if gas_used > gas_limit {
            return Err(PrecompileErrors::Error(Error::OutOfGas));
        }
        Ok(PrecompileOutput {
            bytes: bytes.clone(),
            gas_used,
        })
    }
}
