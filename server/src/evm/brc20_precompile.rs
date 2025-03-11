use db::DB;
use revm::{
    precompile::Error,
    primitives::{Bytes, PrecompileErrors, PrecompileOutput, PrecompileResult},
    ContextStatefulPrecompile, InnerEvmContext,
};
use solabi::{selector, FunctionEncoder};

lazy_static::lazy_static! {
    static ref BRC20_CLIENT: reqwest::blocking::Client = reqwest::blocking::Client::new();
    static ref BRC20_PROG_BALANCE_SERVER_URL: String = std::env::var("BRC20_PROG_BALANCE_SERVER_URL")
            .unwrap_or("http://localhost:18546".to_string());
}

pub struct BRC20Precompile;

const BALANCE_OF: FunctionEncoder<(solabi::Address, String), (solabi::U256,)> =
    FunctionEncoder::new(selector!("balanceOf(address,string)"));

impl ContextStatefulPrecompile<DB> for BRC20Precompile {
    fn call(
        &self,
        bytes: &Bytes,
        gas_limit: u64,
        _evmctx: &mut InnerEvmContext<DB>,
    ) -> PrecompileResult {
        let gas_used = 100000;
        let params = BALANCE_OF.decode_params(&bytes).unwrap();

        let response = BRC20_CLIENT
            .get(&*BRC20_PROG_BALANCE_SERVER_URL)
            .query(&[
                ("address", params.0.to_string()),
                ("ticker", params.1.to_string()),
            ])
            .send()
            .unwrap();

        let balance = response.text().unwrap();
        let balance = balance.parse::<u64>().unwrap_or(0);
        let balance = solabi::U256::from(balance);
        let bytes = BALANCE_OF.encode_returns(&(balance,));

        if gas_used > gas_limit {
            return Err(PrecompileErrors::Error(Error::OutOfGas));
        }
        Ok(PrecompileOutput {
            bytes: Bytes::from(bytes),
            gas_used,
        })
    }
}
