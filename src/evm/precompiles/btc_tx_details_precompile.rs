use alloy_primitives::hex::FromHex;
use alloy_primitives::{FixedBytes, U256};
use alloy_sol_types::{sol, SolCall};
use revm::interpreter::{Gas, InstructionResult, InterpreterResult};
use revm::primitives::Bytes;

use crate::evm::precompiles::btc_utils::{get_block_height, get_raw_transaction};
use crate::evm::precompiles::{precompile_error, precompile_output, use_gas};

static GAS_PER_RPC_CALL: u64 = 100000;

/*
    Signature for the getTxDetails function in the BTCPrecompile contract
    Uses get raw tx details from the blockchain using the json rpc and returns the details from the transaction
    ScriptPubKey for the vin transaction is fetched using the txid and vout

    # Returns (block_height, vin_txids, vin_vouts, vin_scriptPubKeys, vin_values, vout_scriptPubKeys, vout_values) in a tuple
    # Errors - Returns an error if the transaction details are not found
*/
sol! {
    function getTxDetails(bytes32) returns (uint256 block_height, bytes32[] vin_txids, uint256[] vin_vouts , bytes[] vin_scriptPubKeys, uint256[] vin_values, bytes[] vout_scriptPubKeys, uint256[] vout_values);
}

pub fn btc_tx_details_precompile(bytes: &Bytes, gas_limit: u64) -> InterpreterResult {
    let mut interpreter_result =
        InterpreterResult::new(InstructionResult::Stop, Bytes::new(), Gas::new(gas_limit));

    if !use_gas(&mut interpreter_result, GAS_PER_RPC_CALL) {
        return interpreter_result;
    }

    let result = getTxDetailsCall::abi_decode(&bytes, false);

    if result.is_err() {
        // Invalid params
        return precompile_error(interpreter_result);
    }

    let txid = result.unwrap()._0;

    let response = get_raw_transaction(&hex::encode(txid));

    if response["error"].is_object() || !response["result"].is_object() {
        tracing::error!("Error: {}", response["error"]["message"]);
        return precompile_error(interpreter_result);
    }

    let response = response["result"].clone();

    let vin_count = response["vin"].as_array();
    if !use_gas(
        &mut interpreter_result,
        vin_count.unwrap().len() as u64 * GAS_PER_RPC_CALL,
    ) {
        return interpreter_result;
    }

    let block_hash = response["blockhash"].as_str().unwrap_or("").to_string();

    let block_height_result = get_block_height(&block_hash);
    if block_height_result["error"].is_object() || !block_height_result["result"].is_object() {
        return precompile_error(interpreter_result);
    }

    let block_height = Some(block_height_result["result"]["height"].as_u64().unwrap());

    let block_height = U256::from(block_height.unwrap());

    let mut vin_txids = Vec::new();
    let mut vin_vouts = Vec::new();
    let mut vin_script_pub_keys = Vec::new();
    let mut vin_values = Vec::new();
    let mut vout_script_pub_keys = Vec::new();
    let mut vout_values = Vec::new();

    for vin in response["vin"].as_array().unwrap().into_iter() {
        let vin_txid = vin["txid"].as_str().unwrap_or("").to_string();
        let vin_vout = vin["vout"].as_u64().unwrap_or(0);

        // Get the scriptPubKey from the vin transaction, using the txid and vout
        let vin_script_pub_key_response = get_raw_transaction(&vin_txid);
        if vin_script_pub_key_response["error"].is_object()
            || !vin_script_pub_key_response["result"].is_object()
        {
            return precompile_error(interpreter_result);
        }

        let vin_script_pub_key_response = vin_script_pub_key_response["result"].clone();
        let vin_script_pub_key_hex = vin_script_pub_key_response["vout"][vin_vout as usize]
            ["scriptPubKey"]["hex"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let vin_script_pub_key_bytes = Bytes::from(hex::decode(vin_script_pub_key_hex).unwrap());

        let vin_value = vin_script_pub_key_response["vout"][vin_vout as usize]["value"]
            .as_f64()
            .unwrap_or(0.0);
        let vin_value = (vin_value * 100000000.0) as u64;

        vin_txids.push(FixedBytes::from_hex(vin_txid).unwrap());
        vin_vouts.push(U256::from(vin_vout));
        vin_script_pub_keys.push(vin_script_pub_key_bytes);
        vin_values.push(U256::from(vin_value));
    }

    for vout in response["vout"].as_array().unwrap().into_iter() {
        let vout_script_pub_key_hex = vout["scriptPubKey"]["hex"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let vout_script_pub_key_bytes = Bytes::from(hex::decode(vout_script_pub_key_hex).unwrap());
        let vout_value = vout["value"].as_f64().unwrap_or(0.0);
        let vout_value = (vout_value * 100000000.0) as u64;

        vout_script_pub_keys.push(vout_script_pub_key_bytes);
        vout_values.push(U256::from(vout_value));
    }

    let bytes = getTxDetailsCall::abi_encode_returns(&(
        U256::from(block_height),
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
        let response = btc_tx_details_precompile(
            &Bytes::from(getTxDetailsCall::new((txid,)).abi_encode()),
            1000000,
        );

        let returns = getTxDetailsCall::abi_decode_returns(&response.output, false).unwrap();

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
}
