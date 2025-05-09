use std::error::Error;

use alloy_primitives::{Bytes, U256};
use alloy_sol_types::{sol, SolCall};
use revm::interpreter::{Gas, InstructionResult, InterpreterResult};
use ureq::Agent;

use crate::engine::precompiles::{precompile_error, precompile_output, use_gas, PrecompileCall};
use crate::global::CONFIG;

lazy_static::lazy_static! {
    static ref BRC20_CLIENT: Agent = Agent::new_with_defaults();
}

sol! {
    function balanceOf(bytes ticker, bytes pkscript) returns (uint256);
}

pub fn brc20_balance_precompile(call: &PrecompileCall) -> InterpreterResult {
    let mut interpreter_result = InterpreterResult::new(
        InstructionResult::Stop,
        Bytes::new(),
        Gas::new(call.gas_limit),
    );

    if !use_gas(&mut interpreter_result, 100000) {
        return interpreter_result;
    }

    let Ok(inputs) = balanceOfCall::abi_decode(&call.bytes) else {
        return precompile_error(interpreter_result);
    };

    let Ok(balance) = get_brc20_balance(&inputs.ticker, &inputs.pkscript) else {
        return precompile_error(interpreter_result);
    };

    let bytes = balanceOfCall::abi_encode_returns(&U256::from(balance));

    return precompile_output(interpreter_result, bytes);
}

pub fn get_brc20_balance(ticker: &Bytes, pkscript: &Bytes) -> Result<u128, Box<dyn Error>> {
    BRC20_CLIENT
        .get(CONFIG.read().brc20_balance_server_url.as_str())
        .query("ticker", hex::encode(ticker))
        .query("pkscript", hex::encode(pkscript))
        .call()?
        .body_mut()
        .read_to_string()?
        .parse::<u128>()
        .map_err(|e| e.into())
}
