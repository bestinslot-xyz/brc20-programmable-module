use alloy_primitives::Bytes;
use alloy_sol_types::{sol, SolCall};
use bip322::verify_simple;
use bitcoin::consensus::Decodable;
use bitcoin::Witness;
use revm::interpreter::{Gas, InstructionResult, InterpreterResult};

use crate::evm::precompiles::{
    precompile_error, precompile_output, use_gas, PrecompileCall, BITCOIN_NETWORK,
};

sol! {
    function verify(bytes pkscript, bytes message, bytes signature) returns (bool success);
}

pub fn bip322_verify_precompile(call: &PrecompileCall) -> InterpreterResult {
    let mut interpreter_result = InterpreterResult::new(
        InstructionResult::Stop,
        Bytes::new(),
        Gas::new(call.gas_limit),
    );

    if !use_gas(&mut interpreter_result, 100000) {
        return interpreter_result;
    }

    let Ok(inputs) = verifyCall::abi_decode(&call.bytes) else {
        return precompile_error(interpreter_result);
    };

    let (pkscript, message, signature) = (inputs.pkscript, inputs.message, inputs.signature);

    let address = bitcoin::Address::from_script(
        &bitcoin::Script::from_bytes(pkscript.iter().as_slice()),
        *BITCOIN_NETWORK,
    );

    let Ok(address) = address else {
        return precompile_error(interpreter_result);
    };

    let address = address;
    let message = message.iter().as_slice();
    let signature = signature.iter().as_slice();
    let signature = Witness::consensus_decode(&mut signature.iter().as_slice());

    let Ok(signature) = signature else {
        return precompile_error(interpreter_result);
    };

    let Ok(_) = verify_simple(&address, &message, signature) else {
        return precompile_error(interpreter_result);
    };

    return precompile_output(interpreter_result, verifyCall::abi_encode_returns(&true));
}

#[cfg(test)]
mod tests {
    use bitcoin::consensus::Encodable;

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

        let result = bip322_verify_precompile(&PrecompileCall {
            bytes: Bytes::from_iter(bytes.iter()),
            gas_limit: 1000000,
            block_height: 0,
        });

        assert!(result.is_ok());

        let success = verifyCall::abi_decode_returns(&result.output).unwrap();
        assert!(success);
    }
}
