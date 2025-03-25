use bip322::verify_simple_encoded;
use revm::interpreter::{Gas, InstructionResult, InterpreterResult};
use revm::primitives::Bytes;
use solabi::{selector, FunctionEncoder};

use super::precompile_output;
use crate::evm::precompiles::{precompile_error, use_gas};

const VERIFY: FunctionEncoder<(String, String, String), (bool,)> =
    FunctionEncoder::new(selector!("verify(string,string,string)"));

pub fn bip322_verify_precompile(bytes: &Bytes, gas_limit: u64) -> InterpreterResult {
    let mut interpreter_result =
        InterpreterResult::new(InstructionResult::Stop, Bytes::new(), Gas::new(gas_limit));
    let result = VERIFY.decode_params(&bytes);

    if result.is_err() {
        return precompile_error(interpreter_result);
    }

    let (address, message, signature) = result.unwrap();
    let result = verify_simple_encoded(&address, &message, &signature);

    if !use_gas(&mut interpreter_result, 100000) {
        return interpreter_result;
    }

    match result {
        Ok(_) => {
            return precompile_output(interpreter_result, VERIFY.encode_returns(&(true,)));
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

        let bytes = VERIFY.encode_params(&(address.to_string(), message.to_string(), signature));

        let result = bip322_verify_precompile(&Bytes::from_iter(bytes.iter()), 1000000);
        let (success,) = VERIFY.decode_returns(&result.output).unwrap();

        assert!(success);
    }
}
