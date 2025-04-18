use alloy_primitives::{Bytes, FixedBytes, U256};
use alloy_sol_types::{sol, SolCall};
use bitcoin::hashes::Hash;
use revm::interpreter::{Gas, InstructionResult, InterpreterResult};

use crate::evm::precompiles::btc_utils::{get_block_info, get_raw_transaction};
use crate::evm::precompiles::{precompile_error, precompile_output, use_gas, PrecompileCall};

static GAS_PER_RPC_CALL: u64 = 100000;

/*
    Signature for the getTxDetails function in the BTCPrecompile contract
    Uses get raw tx details from the blockchain using the json rpc and returns the details from the transaction
    ScriptPubKey for the vin transaction is fetched using the txid and vout

    # Returns (block_height, vin_txids, vin_vouts, vin_scriptPubKeys, vin_values, vout_scriptPubKeys, vout_values) in a tuple
    # Errors - Returns an error if the transaction details are not found
*/
sol! {
    function getTxDetails(bytes32 txid) returns (uint256 block_height, bytes32[] vin_txids, uint256[] vin_vouts , bytes[] vin_scriptPubKeys, uint256[] vin_values, bytes[] vout_scriptPubKeys, uint256[] vout_values);
}

pub fn btc_tx_details_precompile(call: &PrecompileCall) -> InterpreterResult {
    let mut interpreter_result = InterpreterResult::new(
        InstructionResult::Stop,
        Bytes::new(),
        Gas::new(call.gas_limit),
    );

    if !use_gas(&mut interpreter_result, GAS_PER_RPC_CALL) {
        return interpreter_result;
    }

    let Ok(txid) = getTxDetailsCall::abi_decode(&call.bytes) else {
        return precompile_error(interpreter_result);
    };

    let Ok(raw_tx_info) = get_raw_transaction(&txid.txid) else {
        // Failed to get transaction details
        return precompile_error(interpreter_result);
    };

    if !use_gas(
        &mut interpreter_result,
        raw_tx_info.vin.len() as u64 * GAS_PER_RPC_CALL,
    ) {
        return interpreter_result;
    }

    let Some(block_hash) = raw_tx_info.blockhash else {
        // Failed to get block hash, must be a mempool transaction
        return precompile_error(interpreter_result);
    };

    let Ok(block_info) = get_block_info(&block_hash) else {
        // Failed to get block height
        return precompile_error(interpreter_result);
    };

    if block_info.height > call.block_height as usize {
        // Transaction is in the future, ignore it
        return precompile_error(interpreter_result);
    }

    let mut vin_txids = Vec::new();
    let mut vin_vouts = Vec::new();
    let mut vin_script_pub_keys: Vec<Bytes> = Vec::new();
    let mut vin_values = Vec::new();
    let mut vout_script_pub_keys: Vec<Bytes> = Vec::new();
    let mut vout_values = Vec::new();

    for vin in raw_tx_info.vin {
        let Some(vin_txid) = vin.txid.map(|txid| {
            let mut bytes = FixedBytes::<32>::from_slice(txid.as_raw_hash().as_byte_array());
            bytes.reverse();
            bytes
        }) else {
            // Failed to get vin txid
            return precompile_error(interpreter_result);
        };

        let Some(vin_vout) = vin.vout else {
            // Failed to get vin vout
            return precompile_error(interpreter_result);
        };

        vin_txids.push(vin_txid);
        vin_vouts.push(U256::from(vin_vout));

        // Get the scriptPubKey from the vin transaction, using the txid and vout
        let Ok(vin_transaction) = get_raw_transaction(&vin_txid) else {
            // Failed to get vin transaction details
            return precompile_error(interpreter_result);
        };

        let Some(prev_vout) = &vin_transaction.vout.get(vin_vout as usize) else {
            // Failed to get vin vout
            return precompile_error(interpreter_result);
        };
        vin_script_pub_keys.push(prev_vout.script_pub_key.hex.clone().into());
        vin_values.push(U256::from(prev_vout.value.to_sat()));
    }

    for vout in raw_tx_info.vout {
        vout_script_pub_keys.push(vout.script_pub_key.hex.into());
        vout_values.push(U256::from(vout.value.to_sat()));
    }

    let bytes = getTxDetailsCall::abi_encode_returns_tuple(&(
        U256::from(block_info.height),
        vin_txids,
        vin_vouts,
        vin_script_pub_keys,
        vin_values,
        vout_script_pub_keys,
        vout_values,
    ));

    return precompile_output(interpreter_result, bytes);
}

#[cfg(test)]
mod tests {
    use alloy_primitives::hex::FromHex;

    use super::*;
    use crate::evm::precompiles::btc_utils::skip_btc_tests;

    #[test]
    fn test_get_tx_details_encode_params_single_vin_vout() {
        let txid = hex::decode("d09d26752d0a33d1bdb0213cf36819635d1258a7e4fcbe669e12bc7dab8cecdd")
            .unwrap();
        let data = getTxDetailsCall::new((txid.as_slice().try_into().unwrap(),)).abi_encode();
        assert_eq!(
            hex::encode(data),
            "5579a4a5d09d26752d0a33d1bdb0213cf36819635d1258a7e4fcbe669e12bc7dab8cecdd"
        );
    }

    #[test]
    fn test_get_tx_details_encode_params_multiple_vin_vout() {
        let txid = FixedBytes::from_hex(
            "4183fb733b9553ca8b93208c91dda18bee3d0b8510720b15d76d979af7fd9926",
        )
        .unwrap();
        let data = getTxDetailsCall::new((txid,)).abi_encode();
        assert_eq!(
            hex::encode(data),
            "5579a4a54183fb733b9553ca8b93208c91dda18bee3d0b8510720b15d76d979af7fd9926"
        );
    }

    #[test]
    fn test_get_tx_details_signet() {
        if skip_btc_tests() {
            return;
        }

        // https://mempool.space/signet/tx/d09d26752d0a33d1bdb0213cf36819635d1258a7e4fcbe669e12bc7dab8cecdd
        let txid = FixedBytes::from_hex(
            "d09d26752d0a33d1bdb0213cf36819635d1258a7e4fcbe669e12bc7dab8cecdd",
        )
        .unwrap();
        let response = btc_tx_details_precompile(&PrecompileCall {
            bytes: getTxDetailsCall::new((txid,)).abi_encode().into(),
            gas_limit: 1000000,
            block_height: 240961,
        });

        assert!(response.is_ok());

        let returns = getTxDetailsCall::abi_decode_returns(&response.output).unwrap();

        assert_eq!(returns.block_height, U256::from(240960u64));
        assert_eq!(returns.vin_txids.len(), 1);
        assert_eq!(
            returns.vin_txids[0],
            FixedBytes::from_hex(
                "8d4bc3ac21211723436e35ffbf32a58f74fe942e0ea10936504db07afb1af7c3"
            )
            .unwrap()
        );
        assert_eq!(returns.vin_vouts.len(), 1);
        assert_eq!(returns.vin_vouts[0], U256::from(19u64));
        assert_eq!(returns.vin_scriptPubKeys.len(), 1);
        assert_eq!(
            returns.vin_scriptPubKeys[0],
            Bytes::from_hex("51204a6041f54b8cf8b2d48c6f725cb0514e51e5e7e7ac429c33da62e98765dd62f3")
                .unwrap()
        );
        assert_eq!(returns.vin_values.len(), 1);
        assert_eq!(returns.vin_values[0], U256::from(10000000u64));
        assert_eq!(returns.vout_scriptPubKeys.len(), 1);
        assert_eq!(
            returns.vout_scriptPubKeys[0],
            Bytes::from_hex("0014f477952f33561c1b89a1fe9f28682f623263e159").unwrap()
        );
        assert_eq!(returns.vout_values.len(), 1);
        assert_eq!(returns.vout_values[0], U256::from(9658000u64));
    }

    #[test]
    fn test_get_tx_details_signet_future_transaction() {
        if skip_btc_tests() {
            return;
        }

        // https://mempool.space/signet/tx/d09d26752d0a33d1bdb0213cf36819635d1258a7e4fcbe669e12bc7dab8cecdd
        let txid = FixedBytes::from_hex(
            "d09d26752d0a33d1bdb0213cf36819635d1258a7e4fcbe669e12bc7dab8cecdd",
        )
        .unwrap();
        let response = btc_tx_details_precompile(&PrecompileCall {
            bytes: getTxDetailsCall::new((txid,)).abi_encode().into(),
            gas_limit: 1000000,
            block_height: 240959,
        });

        assert!(response.is_error());
    }
}
