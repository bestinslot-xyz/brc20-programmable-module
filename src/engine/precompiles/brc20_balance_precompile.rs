use alloy::primitives::{Bytes, U256};
use alloy::sol_types::{sol, SolCall};
use revm::interpreter::{Gas, InstructionResult, InterpreterResult};
use ureq::Agent;

use crate::engine::precompiles::{precompile_error, precompile_output, use_gas, PrecompileCall};
use crate::global::{CONFIG, GAS_PER_BRC20_BALANCE_CALL};

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

    if !use_gas(&mut interpreter_result, GAS_PER_BRC20_BALANCE_CALL) {
        return interpreter_result;
    }

    let Ok(inputs) = balanceOfCall::abi_decode(&call.bytes) else {
        return precompile_error(interpreter_result, "Failed to decode parameters");
    };

    let balance = get_brc20_balance(&inputs.ticker, &inputs.pkscript);

    let bytes = balanceOfCall::abi_encode_returns(&U256::from(balance));

    return precompile_output(interpreter_result, bytes);
}

pub fn get_brc20_balance(ticker: &Bytes, pkscript: &Bytes) -> u128 {
    let mut balance_response = match BRC20_CLIENT
        .get(CONFIG.read().brc20_balance_server_url.as_str())
        .query("ticker", hex::encode(ticker))
        .query("pkscript", hex::encode(pkscript))
        .call()
    {
        Ok(response) => response,
        Err(err) => {
            panic!("Error calling BRC20 balance server: {}", err);
        }
    };

    let Ok(balance_string) = balance_response.body_mut().read_to_string() else {
        panic!("Failed to read response body from BRC20 balance server");
    };

    let Ok(balance) = balance_string.parse::<u128>() else {
        panic!("Failed to parse balance string to u128: {}", balance_string);
    };

    balance
}
