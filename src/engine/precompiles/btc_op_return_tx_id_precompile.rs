use alloy::primitives::Bytes;
use alloy::sol_types::{sol, SolCall};
use revm::interpreter::{Gas, InstructionResult, InterpreterResult};

use crate::engine::precompiles::{precompile_output, use_gas, PrecompileCall};
use crate::global::GAS_PER_OP_RETURN_TX_ID;

/*
    Signature for the getTxId function in the BTCPrecompile contract
    Returns the op return transaction id used to call the BRC2.0 programmable module

    # Returns (block_height, vin_txids, vin_vouts, vin_scriptPubKeys, vin_values, vout_scriptPubKeys, vout_values) in a tuple
    # Errors - Returns an error if the transaction details are not found
*/
sol! {
    function getTxId() returns (bytes32);
}

pub fn get_op_return_tx_id_precompile(call: &PrecompileCall) -> InterpreterResult {
    let mut interpreter_result = InterpreterResult::new(
        InstructionResult::Stop,
        Bytes::new(),
        Gas::new(call.gas_limit),
    );

    if !use_gas(&mut interpreter_result, GAS_PER_OP_RETURN_TX_ID) {
        return interpreter_result;
    }

    return precompile_output(
        interpreter_result,
        getTxIdCall::abi_encode_returns(&call.current_op_return_tx_id),
    );
}

#[cfg(test)]
mod tests {
    use revm::primitives::U256;

    use super::*;

    #[test]
    fn test_get_op_return_tx_id_precompile() {
        let bytes: [u8; 32] = [1; 32];
        let result = get_op_return_tx_id_precompile(&PrecompileCall {
            bytes: Bytes::new(),
            gas_limit: 1000000,
            block_height: U256::ZERO,
            current_op_return_tx_id: bytes.into(),
        });

        assert!(result.is_ok());
        assert_eq!(result.output, Bytes::from_iter(bytes.iter()));
    }
}
