use alloy_sol_types::{sol, SolCall};
use bip322::verify_simple_encoded;
use revm::interpreter::{Gas, InstructionResult, InterpreterResult};
use revm::primitives::Bytes;

use super::precompile_output;
use crate::evm::precompiles::{precompile_error, use_gas};

sol! {
    function verify(string, string, string) returns (bool);
}

pub fn bip322_verify_precompile(bytes: &Bytes, gas_limit: u64) -> InterpreterResult {
    let mut interpreter_result =
        InterpreterResult::new(InstructionResult::Stop, Bytes::new(), Gas::new(gas_limit));
    let result = verifyCall::abi_decode(&bytes, false);

    if result.is_err() {
        return precompile_error(interpreter_result);
    }

    let result = result.unwrap();

    let address = result._0;
    let message = result._1;
    let signature = result._2;

    let result = verify_simple_encoded(&address, &message, &signature);

    if !use_gas(&mut interpreter_result, 100000) {
        return interpreter_result;
    }

    match result {
        Ok(_) => {
            return precompile_output(interpreter_result, verifyCall::abi_encode_returns(&(true,)));
        }
        Err(_) => return precompile_error(interpreter_result),
    }
}

#[cfg(test)]
mod tests {
    use revm::primitives::Bytes;

    use super::*;

    #[test]
    fn test_verify() {
        let address = "bc1q9vza2e8x573nczrlzms0wvx3gsqjx7vavgkx0l";
        let message = "Hello World";
        let wif_private_key = "L3VFeEujGtevx9w18HD1fhRbCH67Az2dpCymeRE1SoPK6XQtaN2k";

        let signature = bip322::sign_simple_encoded(&address, &message, &wif_private_key).unwrap();

        let bytes =
            verifyCall::new((address.to_string(), message.to_string(), signature)).abi_encode();

        let result = bip322_verify_precompile(&Bytes::from_iter(bytes.iter()), 1000000);
        let returns = verifyCall::abi_decode_returns(&result.output, false).unwrap();

        assert!(returns._0);
    }
}
