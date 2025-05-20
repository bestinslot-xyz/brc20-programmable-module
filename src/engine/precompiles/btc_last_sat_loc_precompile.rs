use alloy_primitives::{Bytes, FixedBytes, U256};
use alloy_sol_types::{sol, SolCall};
use bitcoin::hashes::Hash;
use revm::interpreter::{Gas, InstructionResult, InterpreterResult};

use crate::engine::precompiles::btc_utils::{
    get_block_height, get_transaction, get_transaction_and_block_hash,
};
use crate::engine::precompiles::{precompile_error, precompile_output, use_gas, PrecompileCall};
use crate::global::GAS_PER_BITCOIN_RPC_CALL;

/*
    Signature for the getLastSatLocation function in the LastSatLocationPrecompile contract
    Uses bitcoin rpc to get the transaction details for the given txid
    Then returns the last sat location for the given vout and sat

    # Returns (txid, vout, sat, old_pkscript, new_pkscript) in a tuple
    # Errors - Returns an error if the transaction details are not found
*/
sol! {
    function getLastSatLocation(bytes32 txid, uint256 vout, uint256 sat) returns (bytes32 last_txid, uint256 last_vout, uint256 last_sat, bytes old_pkscript, bytes new_pkscript);
}

pub fn last_sat_location_precompile(call: &PrecompileCall) -> InterpreterResult {
    let mut interpreter_result = InterpreterResult::new(
        InstructionResult::Stop,
        Bytes::new(),
        Gas::new(call.gas_limit),
    );

    let Ok(inputs) = getLastSatLocationCall::abi_decode(&call.bytes) else {
        return precompile_error(interpreter_result, "Failed to decode parameters");
    };

    let txid = inputs.txid;
    let vout = inputs.vout.as_limbs()[0] as usize;
    let sat = inputs.sat.as_limbs()[0];

    if !use_gas(&mut interpreter_result, *GAS_PER_BITCOIN_RPC_CALL) {
        return interpreter_result;
    }

    let Ok((raw_tx_info, block_hash)) = get_transaction_and_block_hash(&txid) else {
        return precompile_error(interpreter_result, "Failed to get transaction details");
    };

    let Some(block_hash) = block_hash else {
        return precompile_error(interpreter_result, "Failed to get block height");
    };

    let Ok(block_height) = get_block_height(&block_hash) else {
        return precompile_error(interpreter_result, "Failed to get block info");
    };

    if block_height > call.block_height as usize {
        return precompile_error(interpreter_result, "Transaction is in the future");
    }

    if raw_tx_info.is_coinbase() {
        return precompile_error(
            interpreter_result,
            "Coinbase transactions are not supported",
        );
    }
    if raw_tx_info.input.len() == 0 {
        return precompile_error(interpreter_result, "No vin found");
    }

    if raw_tx_info.output.len() < vout {
        return precompile_error(interpreter_result, "Vout index out of bounds");
    }

    if let Some(value) = raw_tx_info.output.get(vout) {
        if value.value.to_sat() < sat {
            return precompile_error(interpreter_result, "Sat value out of bounds");
        }
    } else {
        return precompile_error(interpreter_result, "Invalid response");
    }

    let Ok(new_pkscript) = raw_tx_info
        .tx_out(vout)
        .map(|x| Bytes::from(x.script_pubkey.clone().into_bytes()))
    else {
        return precompile_error(interpreter_result, "Invalid response");
    };

    let mut total_vout_sat_count = 0;
    let mut current_vout_index = 0;
    while current_vout_index < vout {
        let value = raw_tx_info.output[current_vout_index].value.to_sat();
        total_vout_sat_count += value;
        current_vout_index += 1;
    }
    total_vout_sat_count += sat;

    let mut total_vin_sat_count = 0;
    let mut current_vin_index = 0;
    let mut result_vin_txid: FixedBytes<32>;
    let mut result_vin_vout: u32;
    let mut old_pkscript;
    let mut current_vin_value;
    let vin_count = raw_tx_info.input.len();
    loop {
        if raw_tx_info.input[current_vin_index]
            .previous_output
            .is_null()
        {
            return precompile_error(interpreter_result, "Failed to get vin txid");
        };

        if !use_gas(&mut interpreter_result, *GAS_PER_BITCOIN_RPC_CALL) {
            return interpreter_result;
        }

        result_vin_txid = FixedBytes::<32>::from_slice(
            raw_tx_info.input[current_vin_index]
                .previous_output
                .txid
                .as_raw_hash()
                .as_byte_array(),
        );
        result_vin_txid.reverse();

        result_vin_vout = raw_tx_info.input[current_vin_index].previous_output.vout;

        let Ok(vin_response) = get_transaction(&result_vin_txid) else {
            return precompile_error(interpreter_result, "Failed to get vin transaction details");
        };
        let Ok(current_vin) = vin_response.tx_out(result_vin_vout as usize) else {
            return precompile_error(interpreter_result, "Failed to get vin vout");
        };
        current_vin_value = current_vin.value.to_sat();
        old_pkscript = Bytes::from(current_vin.script_pubkey.clone().into_bytes());

        total_vin_sat_count += current_vin_value;
        current_vin_index += 1;
        if total_vin_sat_count >= total_vout_sat_count || current_vin_index >= vin_count {
            break;
        }
    }

    if total_vin_sat_count < total_vout_sat_count {
        return precompile_error(interpreter_result, "Insufficient satoshis in vin");
    }

    let bytes = getLastSatLocationCall::abi_encode_returns_tuple(&(
        result_vin_txid,
        U256::from(result_vin_vout as u64),
        U256::from(total_vout_sat_count - (total_vin_sat_count - current_vin_value)),
        old_pkscript,
        new_pkscript,
    ));

    return precompile_output(interpreter_result, bytes);
}

#[cfg(test)]
mod tests {
    use alloy_primitives::hex::FromHex;

    use super::*;
    use crate::engine::precompiles::validate_bitcoin_rpc_status;

    #[test]
    fn test_get_last_sat_location_encode_params_single_vin_vout() {
        if validate_bitcoin_rpc_status().is_err() {
            return;
        }
        // https://mempool.space/signet/tx/d09d26752d0a33d1bdb0213cf36819635d1258a7e4fcbe669e12bc7dab8cecdd
        let txid = FixedBytes::from_hex(
            "d09d26752d0a33d1bdb0213cf36819635d1258a7e4fcbe669e12bc7dab8cecdd",
        )
        .unwrap();
        let vout = U256::from(0u64);
        let sat = U256::from(100u64);
        let data = getLastSatLocationCall::new((txid, vout, sat)).abi_encode();

        assert_eq!(
            hex::encode(data.iter().as_slice()),
            "2aa29404d09d26752d0a33d1bdb0213cf36819635d1258a7e4fcbe669e12bc7dab8cecdd00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000064"
        );

        // Consider mocking the RPC call to bitcoind
        let result = last_sat_location_precompile(&PrecompileCall {
            bytes: data.into(),
            gas_limit: 1000000,
            block_height: 0,
        });
        let result = result;
        let returns = getLastSatLocationCall::abi_decode_returns(&result.output).unwrap();

        let returns = (
            returns.last_txid,
            returns.last_vout,
            returns.last_sat,
            returns.old_pkscript,
            returns.new_pkscript,
        );

        assert_eq!(result.gas.spent(), 200000);

        assert_eq!(
            returns,
            (
                FixedBytes::from_hex(
                    "8d4bc3ac21211723436e35ffbf32a58f74fe942e0ea10936504db07afb1af7c3"
                )
                .unwrap(),
                U256::from(19u64),
                U256::from(100u64),
                Bytes::from_hex(
                    "51204a6041f54b8cf8b2d48c6f725cb0514e51e5e7e7ac429c33da62e98765dd62f3"
                )
                .unwrap(),
                Bytes::from_hex("0014f477952f33561c1b89a1fe9f28682f623263e159").unwrap(),
            )
        );
    }

    #[test]
    fn test_get_last_sat_location_multiple_vin_vout() {
        if validate_bitcoin_rpc_status().is_err() {
            return;
        }
        // https://mempool.space/signet/tx/4183fb733b9553ca8b93208c91dda18bee3d0b8510720b15d76d979af7fd9926
        let txid = FixedBytes::from_hex(
            "4183fb733b9553ca8b93208c91dda18bee3d0b8510720b15d76d979af7fd9926",
        )
        .unwrap();
        let vout = U256::from(0u64);
        let sat = U256::from(250000u64);
        let data = getLastSatLocationCall::new((txid, vout, sat)).abi_encode();

        // Consider mocking the RPC call to bitcoind
        let result = last_sat_location_precompile(&PrecompileCall {
            bytes: data.into(),
            gas_limit: 1000000,
            block_height: 0,
        });
        let result = result;
        let returns = getLastSatLocationCall::abi_decode_returns(&result.output).unwrap();

        let returns = (
            returns.last_txid,
            returns.last_vout,
            returns.last_sat,
            returns.old_pkscript,
            returns.new_pkscript,
        );

        assert_eq!(result.gas.spent(), 400000);

        assert_eq!(
            returns,
            (
                FixedBytes::from_hex(
                    "423d28032bb7b47d2df4aaa42789d9817d6419f9747fc10343c6fdf3d081ff2b"
                )
                .unwrap(),
                U256::from(0u64),
                U256::from(50000u64),
                Bytes::from_hex(
                    "51205174498f5940118461b4f3006e75dfc0ff140afffc9ac9b2937791a1dc3d17d0"
                )
                .unwrap(),
                Bytes::from_hex(
                    "512050927e29d0d61b2d0e855fd027f7dcfa8d0c7412db9bcb69aeccf34d87c8071d"
                )
                .unwrap(),
            )
        );
    }

    #[test]
    fn test_coinbase_tx() {
        if validate_bitcoin_rpc_status().is_err() {
            return;
        }
        // https://mempool.space/signet/tx/3f6201e955c191e714dcf92240a9dd0eea7c65465f60e4d31f5b6e9fd2003409
        let txid = FixedBytes::from_hex(
            "3f6201e955c191e714dcf92240a9dd0eea7c65465f60e4d31f5b6e9fd2003409",
        )
        .unwrap();
        let vout = U256::from(0u64);
        let sat = U256::from(100u64);
        let data = getLastSatLocationCall::new((txid, vout, sat)).abi_encode();

        // Consider mocking the RPC call to bitcoind
        let result = last_sat_location_precompile(&PrecompileCall {
            bytes: data.into(),
            gas_limit: 1000000,
            block_height: 0,
        });

        assert!(result.is_error());
    }
}
