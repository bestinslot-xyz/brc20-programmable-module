use revm::interpreter::{Gas, InstructionResult, InterpreterResult};
use revm::primitives::Bytes;
use solabi::{selector, FunctionEncoder};

use crate::evm::precompiles::{precompile_error, precompile_output, use_gas};

lazy_static::lazy_static! {
    static ref BRC20_CLIENT: reqwest::blocking::Client = reqwest::blocking::Client::new();
    static ref BRC20_PROG_BALANCE_SERVER_URL: String = std::env::var("BRC20_PROG_BALANCE_SERVER_URL")
            .unwrap_or("http://localhost:18546".to_string());
}

const BALANCE_OF: FunctionEncoder<(String, String), (solabi::U256,)> =
    FunctionEncoder::new(selector!("balanceOf(string,string)"));

pub fn brc20_balance_precompile(bytes: &Bytes, gas_limit: u64) -> InterpreterResult {
    let mut interpreter_result =
        InterpreterResult::new(InstructionResult::Stop, Bytes::new(), Gas::new(gas_limit));

    if !use_gas(&mut interpreter_result, 100000) {
        return interpreter_result;
    }

    let result = BALANCE_OF.decode_params(&bytes);

    if result.is_err() {
        return precompile_error(interpreter_result);
    }

    let (ticker, address) = result.unwrap();

    let response = BRC20_CLIENT
        .get(&*BRC20_PROG_BALANCE_SERVER_URL)
        .query(&[("ticker", ticker), ("address", address)])
        .send()
        .unwrap();

    let balance = response.text().unwrap();
    let balance = balance.parse::<u64>().unwrap_or(0);
    let balance = solabi::U256::from(balance);
    let bytes = BALANCE_OF.encode_returns(&(balance,));

    return precompile_output(interpreter_result, bytes);
}
