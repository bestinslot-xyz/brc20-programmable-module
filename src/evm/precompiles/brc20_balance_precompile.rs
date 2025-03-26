use alloy_primitives::U256;
use alloy_sol_types::{sol, SolCall};
use revm::interpreter::{Gas, InstructionResult, InterpreterResult};
use revm::primitives::Bytes;

use crate::evm::precompiles::{precompile_error, precompile_output, use_gas};

lazy_static::lazy_static! {
    static ref BRC20_PROG_BALANCE_SERVER_URL: String = std::env::var("BRC20_PROG_BALANCE_SERVER_URL")
            .unwrap_or("http://localhost:18546".to_string());
}

sol! {
    function balanceOf(string, string) returns (uint256);
}

pub fn brc20_balance_precompile(bytes: &Bytes, gas_limit: u64) -> InterpreterResult {
    let mut interpreter_result =
        InterpreterResult::new(InstructionResult::Stop, Bytes::new(), Gas::new(gas_limit));

    if !use_gas(&mut interpreter_result, 100000) {
        return interpreter_result;
    }

    let result = balanceOfCall::abi_decode(bytes, false);

    if result.is_err() {
        return precompile_error(interpreter_result);
    }

    let returns = result.unwrap();

    let ticker = returns._0;
    let address = returns._1;

    let balance = get_brc20_balance(&ticker, &address);

    if balance.is_err() {
        return precompile_error(interpreter_result);
    }

    let balance = U256::from(balance.unwrap());
    let bytes = balanceOfCall::abi_encode_returns(&(balance,));

    return precompile_output(interpreter_result, bytes);
}

pub fn get_brc20_balance(ticker: &str, address: &str) -> Result<u64, String> {
    let response = ureq::get(BRC20_PROG_BALANCE_SERVER_URL.as_str())
        .query("ticker", ticker)
        .query("address", address)
        .call();

    if response.is_err() {
        return Err("Failed to get balance".into());
    }

    let balance = response.unwrap().body_mut().read_to_string();

    if balance.is_err() {
        return Err("Failed to get balance".into());
    }

    let balance = balance.unwrap().parse::<u64>();

    if balance.is_err() {
        return Err("Failed to get balance".into());
    }

    Ok(balance.unwrap())
}
