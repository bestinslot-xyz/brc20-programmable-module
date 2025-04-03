use std::error::Error;

use alloy_primitives::U256;
use alloy_sol_types::{sol, SolCall};
use revm::interpreter::{Gas, InstructionResult, InterpreterResult};
use revm::primitives::Bytes;
use ureq::Agent;

use crate::evm::precompiles::{precompile_error, precompile_output, use_gas};

lazy_static::lazy_static! {
    static ref BRC20_CLIENT: Agent = Agent::new_with_defaults();
    static ref BRC20_PROG_BALANCE_SERVER_URL: String = std::env::var("BRC20_PROG_BALANCE_SERVER_URL")
            .unwrap_or("http://localhost:18546".to_string());
}

sol! {
    function balanceOf(bytes ticker, bytes pkscript) returns (uint256);
}

pub fn brc20_balance_precompile(bytes: &Bytes, gas_limit: u64) -> InterpreterResult {
    let mut interpreter_result =
        InterpreterResult::new(InstructionResult::Stop, Bytes::new(), Gas::new(gas_limit));

    if !use_gas(&mut interpreter_result, 100000) {
        return interpreter_result;
    }

    let result = balanceOfCall::abi_decode(bytes, false);

    let Ok(returns) = result else {
        return precompile_error(interpreter_result);
    };

    let ticker = returns.ticker;
    let pkscript = returns.pkscript;

    let Ok(balance) = get_brc20_balance(&ticker, &pkscript) else {
        return precompile_error(interpreter_result);
    };

    let balance = U256::from(balance);
    let bytes = balanceOfCall::abi_encode_returns(&(balance,));

    return precompile_output(interpreter_result, bytes);
}

pub fn get_brc20_balance(ticker: &Bytes, pkscript: &Bytes) -> Result<u128, Box<dyn Error>> {
    BRC20_CLIENT
        .get(BRC20_PROG_BALANCE_SERVER_URL.as_str())
        .query("ticker", hex::encode(ticker))
        .query("pkscript", hex::encode(pkscript))
        .call()?
        .body_mut()
        .read_to_string()?
        .parse::<u128>()
        .map_err(|e| e.into())
}
