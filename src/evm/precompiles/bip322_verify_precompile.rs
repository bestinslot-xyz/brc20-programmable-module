use alloy_sol_types::{sol, SolCall};
use bip322::verify_simple;
use bitcoin::consensus::Decodable;
use bitcoin::Witness;
use revm::interpreter::{Gas, InstructionResult, InterpreterResult};
use revm::primitives::Bytes;

use crate::evm::precompiles::{precompile_error, precompile_output, use_gas, BITCOIN_NETWORK};

sol! {
    function verify(bytes pkscript, bytes message, bytes signature) returns (bool success);
}

pub fn bip322_verify_precompile(bytes: &Bytes, gas_limit: u64) -> InterpreterResult {
    let mut interpreter_result =
        InterpreterResult::new(InstructionResult::Stop, Bytes::new(), Gas::new(gas_limit));

    if !use_gas(&mut interpreter_result, 100000) {
        return interpreter_result;
    }

    let result = verifyCall::abi_decode(&bytes, false);

    if result.is_err() {
        return precompile_error(interpreter_result);
    }

    let result = result.unwrap();

    let (pkscript, message, signature) = (result.pkscript, result.message, result.signature);

    let address = bitcoin::Address::from_script(
        &bitcoin::Script::from_bytes(pkscript.iter().as_slice()),
        *BITCOIN_NETWORK,
    );

    if address.is_err() {
        // Invalid pkscript
        return precompile_error(interpreter_result);
    }

    let address = address.unwrap();
    let message = message.iter().as_slice();
    let signature = signature.iter().as_slice();
    let signature = Witness::consensus_decode(&mut signature.iter().as_slice());

    if signature.is_err() {
        // Invalid signature
        return precompile_error(interpreter_result);
    }

    let signature = signature.unwrap();

    let result = verify_simple(&address, &message, signature);

    match result {
        Ok(_) => {
            return precompile_output(interpreter_result, verifyCall::abi_encode_returns(&(true,)));
        }
        Err(_) => return precompile_error(interpreter_result),
    }
}

#[cfg(test)]
mod tests {
    use bitcoin::consensus::Encodable;
    use revm::primitives::Bytes;

    use super::*;

    #[test]
    fn test_verify() {
        let pkscript =
            Bytes::from(hex::decode("00142b05d564e6a7a33c087f16e0f730d1440123799d").unwrap());

        let address = bitcoin::Address::from_script(
            &bitcoin::Script::from_bytes(pkscript.iter().as_slice()),
            bitcoin::Network::Signet,
        )
        .unwrap();

        let message = Bytes::from("Hello World".as_bytes());

        let wif_private_key = "L3VFeEujGtevx9w18HD1fhRbCH67Az2dpCymeRE1SoPK6XQtaN2k";
        let wif_private_key = bitcoin::PrivateKey::from_wif(wif_private_key).unwrap();

        let signature = bip322::sign_simple(&address, &message, wif_private_key).unwrap();

        let mut signature_bytes = Vec::new();
        signature.consensus_encode(&mut signature_bytes).unwrap();

        let bytes = verifyCall::new((
            pkscript.clone(),
            message.clone(),
            Bytes::from(signature_bytes.clone()),
        ))
        .abi_encode();

        let result = bip322_verify_precompile(&Bytes::from_iter(bytes.iter()), 1000000);

        assert!(result.is_ok());

        let returns = verifyCall::abi_decode_returns(&result.output, false).unwrap();
        assert!(returns.success);
    }
}
