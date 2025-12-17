use alloy::primitives::{Bytes, FixedBytes, U256};
use alloy::sol_types::{sol, SolCall};
use bitcoin::hashes::Hash;
use revm::interpreter::{Gas, InstructionResult, InterpreterResult};

use crate::engine::precompiles::btc_utils::{
    get_block_height, get_transaction_and_block_hash_with_overrides, get_transaction_with_overrides,
};
use crate::engine::precompiles::{precompile_error, precompile_output, use_gas, PrecompileCall};
use crate::global::GAS_PER_BITCOIN_RPC_CALL;

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

    if !use_gas(&mut interpreter_result, GAS_PER_BITCOIN_RPC_CALL) {
        return interpreter_result;
    }

    let Ok(txid) = getTxDetailsCall::abi_decode(&call.bytes) else {
        return precompile_error(interpreter_result, "Failed to decode parameters");
    };

    let Ok((tx_info, block_hash)) =
        get_transaction_and_block_hash_with_overrides(&txid.txid, &call.btc_tx_hexes_data)
    else {
        return precompile_error(interpreter_result, "Failed to get transaction details");
    };

    if !use_gas(
        &mut interpreter_result,
        // +1 for block height retrieval
        (tx_info.input.len()) as u64 * GAS_PER_BITCOIN_RPC_CALL,
    ) {
        return interpreter_result;
    }

    let Some(block_hash) = block_hash else {
        return precompile_error(interpreter_result, "Transaction is not confirmed");
    };

    let Ok(block_height) = get_block_height(&block_hash) else {
        return precompile_error(interpreter_result, "Failed to get block info");
    };

    if U256::from(block_height) > call.block_height {
        return precompile_error(interpreter_result, "Transaction is in the future");
    }

    let mut vin_txids = Vec::new();
    let mut vin_vouts = Vec::new();
    let mut vin_script_pub_keys: Vec<Bytes> = Vec::new();
    let mut vin_values = Vec::new();
    let mut vout_script_pub_keys: Vec<Bytes> = Vec::new();
    let mut vout_values = Vec::new();

    for vin in tx_info.input {
        if vin.previous_output.is_null() {
            return precompile_error(interpreter_result, "Failed to get vin txid");
        };

        let mut vin_txid =
            FixedBytes::<32>::from_slice(vin.previous_output.txid.as_raw_hash().as_byte_array());
        vin_txid.reverse();

        vin_txids.push(vin_txid);
        vin_vouts.push(U256::from(vin.previous_output.vout));

        // Get the scriptPubKey from the vin transaction, using the txid and vout
        let Ok(vin_transaction) =
            get_transaction_with_overrides(&vin_txid, &call.btc_tx_hexes_data)
        else {
            return precompile_error(interpreter_result, "Failed to get vin transaction details");
        };

        let Some(prev_vout) = &vin_transaction
            .output
            .get(vin.previous_output.vout as usize)
        else {
            return precompile_error(interpreter_result, "Failed to get vin vout");
        };
        vin_script_pub_keys.push(prev_vout.script_pubkey.clone().into_bytes().into());
        vin_values.push(U256::from(prev_vout.value.to_sat()));
    }

    for vout in tx_info.output {
        vout_script_pub_keys.push(vout.script_pubkey.clone().into_bytes().into());
        vout_values.push(U256::from(vout.value.to_sat()));
    }

    let bytes = getTxDetailsCall::abi_encode_returns_tuple(&(
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
    use std::collections::HashMap;

    use alloy::primitives::hex::FromHex;

    use super::*;
    use crate::engine::precompiles::btc_utils::validate_bitcoin_rpc_status;

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
        if validate_bitcoin_rpc_status().is_err() {
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
            block_height: U256::from(240961u64),
            current_op_return_tx_id: [0u8; 32].into(),
            btc_tx_hexes_data: HashMap::new(),
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
        if validate_bitcoin_rpc_status().is_err() {
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
            block_height: U256::from(240959u64),
            current_op_return_tx_id: [0u8; 32].into(),
            btc_tx_hexes_data: HashMap::new(),
        });

        assert!(response.is_error());
    }

    #[test]
    fn test_overrides_get_tx_details() {
        // Transaction with 2 vins and 2 vouts
        let tx_hex = hex::decode("020000000180b98c54dbab5106d5a1449f4e5fdb9146deca1d48e93d666c5d9290b7c37a3f010000006b483045022100f0e32ceb205a5056694611afcffe4c1f0e63e9c57382607045ff2c3d9b5b7b3f0220111f0323e56d7462a9299833166569f1a68e1f5090b49bea64f541c494109c6c012102d0648f06a31d47112f1ff7848c85ce54b772c513bc3337c98f081c19d3dca260ffffffff02006d7c4d000000001976a91474d463a046e3175142464740bad692fa0762a93e88accad5e5f1b50000001976a914c98fc6bd9c2fd88533f28e6797cfa2a0a0e18ecf88ac00000000")
            .unwrap();

        let txid = FixedBytes::from_hex(
            "704fcd7f85e2ce234252c3d9b20a66096dd52a8c6a209fa0241106ee5aa997c9",
        )
        .unwrap();

        let prev_tx_hex = hex::decode("02000000076891aa93ec3fa9af4c659f0a3a39ff79d9670b858cd804523ff0032e4e25a5e4010000006b483045022100815a47df83dca558e5ba38b1f991ca19fe278ea00086b17f5ed0ed33d037967f0220377dcbb579c6c2898a0847c7dd232d79410179bfc89c135cb655ec93ffc65a41012102d0648f06a31d47112f1ff7848c85ce54b772c513bc3337c98f081c19d3dca260ffffffff8ade7c64df0611c74cf08f680d767e2d37e1e037fea6cc3795fd9e8d79aaebf2180000006a473044022042321d1f203f0417f1be339a168b1ee405677f67584a73252f054448466593ba0220671881321b81dc15c30a74b1c8c39137083e6b8c0fd3efe9f1771e89d7ae50c2012102d0648f06a31d47112f1ff7848c85ce54b772c513bc3337c98f081c19d3dca260ffffffff3c7439a3f5517a915b0d28c109d316440703a5b3f732abc8df81fcad3edefe90020000006b483045022100e998b992894a32d98286b7fc7ebd6356817434437f381c4421e3804f89886a580220474bd2c38056a4fdd47f71125626c0de686df61b09c9c1a1b4463b77027baddf012102d0648f06a31d47112f1ff7848c85ce54b772c513bc3337c98f081c19d3dca260ffffffff5fb5490642ef0dc2e0150a38684cf75f7ef41ec665972aea63c4aec3ebafc51d010000006a473044022018a6a1652596bb6aaf8a6fd4af285e6ce219b39850b99fb446788fd24092753e022037ff2768bb9f9dbdd05475b16b346df1523e832d41c346814fb6d90bccf0316f012102d0648f06a31d47112f1ff7848c85ce54b772c513bc3337c98f081c19d3dca260ffffffffebe7bdc7a285a5cb4794a219ed21d2a5347a2830634912ed8075753ae01dddbd000000006b483045022100edc69ccb9f82c0a43f19c8583757b06191d6a558c0de6827144637d7e178491402205133b28791ffd59b3ecd25a74341d61cd11108ef2f71425b7f777715f5d70ff6012102d0648f06a31d47112f1ff7848c85ce54b772c513bc3337c98f081c19d3dca260ffffffff71314607c169eaac031cbe189115a2165bfadb3a0822dab22808c8c131ab7a810b0000006b4830450221008add5186b4700553869214445f18b46f2f65e9d6c441124bfff3c4fe1013a885022029b9e8e73e4e0bcce40a1ea682a726f9ba6534da5b5d330a2eaaf813b0c39c82012102d0648f06a31d47112f1ff7848c85ce54b772c513bc3337c98f081c19d3dca260ffffffff21c893e571a6e5a5e763ddff7c0fced111ebd1db7fba1e749cb0f9bfe9f6d5ef000000006a4730440220112c9a6347e6bd6f1820619bcccab4cf159f83784c49a18a55649cd577c833e802207974b0e931ca59fc8a2963b68ed06ee1914d591e5eb99caf1fa08de91fd8b7c2012102d0648f06a31d47112f1ff7848c85ce54b772c513bc3337c98f081c19d3dca260ffffffff02002f68590000000017a91408b703915788341bf8669bade82ff204934f478f877051623fb60000001976a914c98fc6bd9c2fd88533f28e6797cfa2a0a0e18ecf88ac00000000")
            .unwrap();

        let prev_tx_id = FixedBytes::from_hex(
            "3f7ac3b790925d6c663de9481dcade4691db5f4e9f44a1d50651abdb548cb980",
        )
        .unwrap();

        let mut btc_tx_hexes = HashMap::new();
        btc_tx_hexes.insert(txid, tx_hex.into());
        btc_tx_hexes.insert(prev_tx_id, prev_tx_hex.into());

        let response = btc_tx_details_precompile(&PrecompileCall {
            bytes: getTxDetailsCall::new((txid,)).abi_encode().into(),
            gas_limit: 1000000,
            block_height: U256::from(0u64),
            current_op_return_tx_id: [0u8; 32].into(),
            btc_tx_hexes_data: btc_tx_hexes,
        });

        assert!(response.is_ok());

        let returns = getTxDetailsCall::abi_decode_returns(&response.output).unwrap();

        assert_eq!(returns.block_height, U256::from(0u64));
        assert_eq!(returns.vin_txids.len(), 1);
        assert_eq!(
            returns.vin_txids[0],
            FixedBytes::from_hex(
                "3f7ac3b790925d6c663de9481dcade4691db5f4e9f44a1d50651abdb548cb980"
            )
            .unwrap()
        );
        assert_eq!(returns.vin_vouts.len(), 1);
        assert_eq!(returns.vin_vouts[0], U256::from(1u64));
        assert_eq!(returns.vin_scriptPubKeys.len(), 1);
        assert_eq!(
            returns.vin_scriptPubKeys[0],
            Bytes::from_hex("76a914c98fc6bd9c2fd88533f28e6797cfa2a0a0e18ecf88ac").unwrap()
        );
        assert_eq!(returns.vin_values.len(), 1);
        assert_eq!(returns.vin_values[0], U256::from(782747455856u64));
        assert_eq!(returns.vout_scriptPubKeys.len(), 2);
        assert_eq!(
            returns.vout_scriptPubKeys[0],
            Bytes::from_hex("76a91474d463a046e3175142464740bad692fa0762a93e88ac").unwrap()
        );
        assert_eq!(
            returns.vout_scriptPubKeys[1],
            Bytes::from_hex("76a914c98fc6bd9c2fd88533f28e6797cfa2a0a0e18ecf88ac").unwrap()
        );
        assert_eq!(returns.vout_values.len(), 2);
        assert_eq!(returns.vout_values[0], U256::from(1300000000u64));
        assert_eq!(returns.vout_values[1], U256::from(781447452106u64));
    }
}
