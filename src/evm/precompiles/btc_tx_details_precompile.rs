use alloy_primitives::U256;
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

    # Returns (block_height, vin_txid, vin_vout, vin_scriptPubKey_hex, vin_value, vout_scriptPubKey_hex, vout_value) in a tuple
    # Errors - Returns an error if the transaction details are not found
*/
sol! {
    function getTxDetails(string) returns (uint256, string[], uint256[], string[], uint256[], string[], uint256[]);
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

    let response = get_raw_transaction(&txid);

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
    let mut vin_script_pub_key_hexes = Vec::new();
    let mut vin_values = Vec::new();
    let mut vout_script_pub_key_hexes = Vec::new();
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

        let vin_value = vin_script_pub_key_response["vout"][vin_vout as usize]["value"]
            .as_f64()
            .unwrap_or(0.0);
        let vin_value = (vin_value * 100000000.0) as u64;

        vin_txids.push(vin_txid);
        vin_vouts.push(U256::from(vin_vout));
        vin_script_pub_key_hexes.push(vin_script_pub_key_hex);
        vin_values.push(U256::from(vin_value));
    }

    for vout in response["vout"].as_array().unwrap().into_iter() {
        let vout_script_pub_key_hex = vout["scriptPubKey"]["hex"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let vout_value = vout["value"].as_f64().unwrap_or(0.0);
        let vout_value = (vout_value * 100000000.0) as u64;

        vout_script_pub_key_hexes.push(vout_script_pub_key_hex);
        vout_values.push(U256::from(vout_value));
    }

    let bytes = getTxDetailsCall::abi_encode_returns(&(
        U256::from(block_height),
        vin_txids,
        vin_vouts,
        vin_script_pub_key_hexes,
        vin_values,
        vout_script_pub_key_hexes,
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
        let txid = "d09d26752d0a33d1bdb0213cf36819635d1258a7e4fcbe669e12bc7dab8cecdd";
        let data = getTxDetailsCall::new((txid.to_string(),)).abi_encode();
        assert_eq!(
            hex::encode(data),
            "963273230000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000004064303964323637353264306133336431626462303231336366333638313936333564313235386137653466636265363639653132626337646162386365636464"
        );
    }

    #[test]
    fn test_get_tx_details_decode_returns_single_vin_vout() {
        // https://mempool.space/testnet4/tx/cedfb4b62224a4782a4453dff73f3d48bb0d7da4d0f2238b0e949f9342de038a
        let data = "0000000000000000000000000000000000000000000000000000000000011f2d00000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001c0000000000000000000000000000000000000000000000000000000000000028000000000000000000000000000000000000000000000000000000000000002c000000000000000000000000000000000000000000000000000000000000003800000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000403366633564343962616439326131626331306339623039383166313363343736303061663837316464633962616331356432326432633035623661653830663100000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000044353132303135346137636266383361356164393239653232343732356239313563613563643763613737313961396261326166393039343561373465333433313933346400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000001e70000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000443531323066636463356137626436366234643361386339316631613163663934616437643536316633613330346266313866616635363738623165653437653738336237000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000014a";
        let returns =
            getTxDetailsCall::abi_decode_returns(hex::decode(data).unwrap().as_slice(), false)
                .unwrap();

        let (
            block_height,
            vin_txids,
            vin_vouts,
            vin_script_pub_key_hexes,
            vin_values,
            vout_script_pub_key_hexes,
            vout_values,
        ) = (
            returns._0, returns._1, returns._2, returns._3, returns._4, returns._5, returns._6,
        );

        assert_eq!(block_height, U256::from(73517u64));
        assert_eq!(vin_txids.len(), 1);
        assert_eq!(vin_vouts.len(), 1);
        assert_eq!(vin_script_pub_key_hexes.len(), 1);
        assert_eq!(vin_values.len(), 1);

        assert_eq!(vout_script_pub_key_hexes.len(), 1);
        assert_eq!(vout_values.len(), 1);

        assert_eq!(
            vin_txids[0],
            "3fc5d49bad92a1bc10c9b0981f13c47600af871ddc9bac15d22d2c05b6ae80f1"
        );
        assert_eq!(vin_vouts[0], U256::from(0u64));
        assert_eq!(
            vin_script_pub_key_hexes[0],
            "5120154a7cbf83a5ad929e224725b915ca5cd7ca7719a9ba2af90945a74e3431934d"
        );
        assert_eq!(vin_values[0], U256::from(487u64));

        assert_eq!(
            vout_script_pub_key_hexes[0],
            "5120fcdc5a7bd66b4d3a8c91f1a1cf94ad7d561f3a304bf18faf5678b1ee47e783b7"
        );
        assert_eq!(vout_values[0], U256::from(330u64));
    }

    #[test]
    fn test_get_tx_details_encode_params_multiple_vin_vout() {
        let txid = "4183fb733b9553ca8b93208c91dda18bee3d0b8510720b15d76d979af7fd9926";
        let data = getTxDetailsCall::new((txid.to_string(),)).abi_encode();
        assert_eq!(
            hex::encode(data),
            "963273230000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000004034313833666237333362393535336361386239333230386339316464613138626565336430623835313037323062313564373664393739616637666439393236"
        );
    }

    #[test]
    fn test_get_tx_details_decode_returns_multiple_vin_vout() {
        // https://mempool.space/testnet4/tx/ce1d2d142eb12fa4fbbb2c361c286483e5c74ca67640496de23beb5ee56d0406
        let data = "0000000000000000000000000000000000000000000000000000000000011d7600000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000028000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000058000000000000000000000000000000000000000000000000000000000000007e00000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000000403162613331333332323662313866313865636339653333333539613063356131323433323033613734666334666466383839386639653730396138323630616100000000000000000000000000000000000000000000000000000000000000406561326135353337343733336433633336313432373235623366343538353762663464373862323931353365613464636466386162356238383062343934663800000000000000000000000000000000000000000000000000000000000000406233663138663062343139656335653435323731363737373633343930656637626532653662346264353531396462613932346564316333316537383764346400000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000160000000000000000000000000000000000000000000000000000000000000004435313230353436656231386135643435396262353964393637396665386638643539386662663735363862663035636464613361663662323631386238666438633366340000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000443531323035343665623138613564343539626235396439363739666538663864353938666266373536386266303563646461336166366232363138623866643863336634000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000044353132303534366562313861356434353962623539643936373966653866386435393866626637353638626630356364646133616636623236313862386664386333663400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000002220000000000000000000000000000000000000000000000000000000000000222000000000000000000000000000000000000000000000000000000000012cb220000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001c0000000000000000000000000000000000000000000000000000000000000001c36613564306231363031303065393964303431656633383436613032000000000000000000000000000000000000000000000000000000000000000000000044353132303534366562313861356434353962623539643936373966653866386435393866626637353638626630356364646133616636623236313862386664386333663400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004435313230623430633036356266636335393632653137303266303964653161356432646663306137323336626261663563313637323532396234313462336565346366350000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000443531323035343665623138613564343539626235396439363739666538663864353938666266373536386266303563646461336166366232363138623866643863336634000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002220000000000000000000000000000000000000000000000000000000000000222000000000000000000000000000000000000000000000000000000000012b211";
        let returns =
            getTxDetailsCall::abi_decode_returns(hex::decode(data).unwrap().as_slice(), false)
                .unwrap();

        let (
            block_height,
            vin_txids,
            vin_vouts,
            vin_script_pub_key_hexes,
            vin_values,
            vout_script_pub_key_hexes,
            vout_values,
        ) = (
            returns._0, returns._1, returns._2, returns._3, returns._4, returns._5, returns._6,
        );

        assert_eq!(block_height, U256::from(73078u64));
        assert_eq!(vin_txids.len(), 3);
        assert_eq!(vin_vouts.len(), 3);
        assert_eq!(vin_script_pub_key_hexes.len(), 3);
        assert_eq!(vin_values.len(), 3);

        assert_eq!(vout_script_pub_key_hexes.len(), 4);
        assert_eq!(vout_values.len(), 4);

        assert_eq!(
            vin_txids[0],
            "1ba3133226b18f18ecc9e33359a0c5a1243203a74fc4fdf8898f9e709a8260aa"
        );
        assert_eq!(vin_vouts[0], U256::from(2u64));
        assert_eq!(
            vin_script_pub_key_hexes[0],
            "5120546eb18a5d459bb59d9679fe8f8d598fbf7568bf05cdda3af6b2618b8fd8c3f4"
        );
        assert_eq!(vin_values[0], U256::from(546u64));

        assert_eq!(
            vin_txids[1],
            "ea2a55374733d3c36142725b3f45857bf4d78b29153ea4dcdf8ab5b880b494f8"
        );
        assert_eq!(vin_vouts[1], U256::from(2u64));
        assert_eq!(
            vin_script_pub_key_hexes[1],
            "5120546eb18a5d459bb59d9679fe8f8d598fbf7568bf05cdda3af6b2618b8fd8c3f4"
        );
        assert_eq!(vin_values[1], U256::from(546u64));

        assert_eq!(
            vin_txids[2],
            "b3f18f0b419ec5e45271677763490ef7be2e6b4bd5519dba924ed1c31e787d4d"
        );
        assert_eq!(vin_vouts[2], U256::from(0u64));
        assert_eq!(
            vin_script_pub_key_hexes[2],
            "5120546eb18a5d459bb59d9679fe8f8d598fbf7568bf05cdda3af6b2618b8fd8c3f4"
        );
        assert_eq!(vin_values[2], U256::from(1231650u64));

        assert_eq!(vout_script_pub_key_hexes[0], "6a5d0b160100e99d041ef3846a02");
        assert_eq!(vout_values[0], U256::from(0u64));
        assert_eq!(
            vout_script_pub_key_hexes[1],
            "5120546eb18a5d459bb59d9679fe8f8d598fbf7568bf05cdda3af6b2618b8fd8c3f4"
        );
        assert_eq!(vout_values[1], U256::from(546u64));
        assert_eq!(
            vout_script_pub_key_hexes[2],
            "5120b40c065bfcc5962e1702f09de1a5d2dfc0a7236bbaf5c1672529b414b3ee4cf5"
        );
        assert_eq!(vout_values[2], U256::from(546u64));
        assert_eq!(
            vout_script_pub_key_hexes[3],
            "5120546eb18a5d459bb59d9679fe8f8d598fbf7568bf05cdda3af6b2618b8fd8c3f4"
        );
        assert_eq!(vout_values[3], U256::from(1225233u64));
    }

    #[test]
    fn test_get_tx_details_signet() {
        if skip_btc_tests() {
            return;
        }

        // https://mempool.space/signet/tx/d09d26752d0a33d1bdb0213cf36819635d1258a7e4fcbe669e12bc7dab8cecdd
        let txid = "d09d26752d0a33d1bdb0213cf36819635d1258a7e4fcbe669e12bc7dab8cecdd";
        let response = btc_tx_details_precompile(
            &Bytes::from(getTxDetailsCall::new((txid.to_string(),)).abi_encode()),
            1000000,
        );

        let returns = getTxDetailsCall::abi_decode_returns(&response.output, false).unwrap();

        let (
            block_height,
            vin_txids,
            vin_vouts,
            vin_script_pub_key_hexes,
            vin_values,
            vout_script_pub_key_hexes,
            vout_values,
        ) = (
            returns._0, returns._1, returns._2, returns._3, returns._4, returns._5, returns._6,
        );

        assert_eq!(block_height, U256::from(240960u64));
        assert_eq!(vin_txids.len(), 1);
        assert_eq!(
            vin_txids[0],
            "8d4bc3ac21211723436e35ffbf32a58f74fe942e0ea10936504db07afb1af7c3"
        );
        assert_eq!(vin_vouts.len(), 1);
        assert_eq!(vin_vouts[0], U256::from(19u64));
        assert_eq!(vin_script_pub_key_hexes.len(), 1);
        assert_eq!(
            vin_script_pub_key_hexes[0],
            "51204a6041f54b8cf8b2d48c6f725cb0514e51e5e7e7ac429c33da62e98765dd62f3"
        );
        assert_eq!(vin_values.len(), 1);
        assert_eq!(vin_values[0], U256::from(10000000u64));
        assert_eq!(vout_script_pub_key_hexes.len(), 1);
        assert_eq!(
            vout_script_pub_key_hexes[0],
            "0014f477952f33561c1b89a1fe9f28682f623263e159"
        );
        assert_eq!(vout_values.len(), 1);
        assert_eq!(vout_values[0], U256::from(9658000u64));
    }
}
