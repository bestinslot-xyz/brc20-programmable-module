use db::DB;
use revm::{
    precompile::Error, primitives::{
        Bytes, PrecompileErrors, PrecompileOutput, PrecompileResult,
    }, ContextStatefulPrecompile, InnerEvmContext
};
use solabi::{selector, FunctionEncoder};

pub struct BRC20Precompile;

const BALANCE_OF: FunctionEncoder<(solabi::Address, String), (solabi::U256,)> =
    FunctionEncoder::new(selector!("verify(address,string)"));

impl ContextStatefulPrecompile<DB> for BRC20Precompile {
    fn call(
        &self,
        bytes: &Bytes,
        gas_limit: u64,
        _evmctx: &mut InnerEvmContext<DB>,
    ) -> PrecompileResult {
        let gas_used = 100000;
        let params = BALANCE_OF.decode_params(&bytes).unwrap();

        // TODO: Implement the actual logic

        if gas_used > gas_limit {
            return Err(PrecompileErrors::Error(Error::OutOfGas));
        }
        Ok(PrecompileOutput {
            bytes: bytes.clone(),
            gas_used,
        })
    }
}
