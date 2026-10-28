use alloy::primitives::Bytes;
use alloy::sol_types::{sol, SolCall};
use bitcoin::consensus::Decodable;
use bitcoin::{Address, Witness};
use revm::interpreter::{Gas, InstructionResult, InterpreterResult};

use crate::engine::hardforks::use_bip322_v0_0_11;
use crate::engine::precompiles::bip322_legacy;
use crate::engine::precompiles::btc_utils::get_bitcoin_network;
use crate::engine::precompiles::{precompile_error, precompile_output, use_gas, PrecompileCall};
use crate::global::GAS_PER_BIP_322_VERIFY;

sol! {
    function verify(bytes pkscript, bytes message, bytes signature) returns (bool success);
}

pub fn bip322_verify_precompile(call: &PrecompileCall) -> InterpreterResult {
    let mut interpreter_result = InterpreterResult::new(
        InstructionResult::Stop,
        Bytes::new(),
        Gas::new(call.gas_limit),
    );

    if !use_gas(&mut interpreter_result, GAS_PER_BIP_322_VERIFY) {
        return interpreter_result;
    }

    // Check bytes length
    if call.bytes.len() > 32768 {
        return precompile_error(
            interpreter_result,
            "Input bytes length exceeds maximum limit of 32 KB",
        );
    }

    let Ok(inputs) = verifyCall::abi_decode(&call.bytes) else {
        return precompile_error(interpreter_result, "Failed to decode parameters");
    };

    let (pkscript, message, signature) = (inputs.pkscript, inputs.message, inputs.signature);

    let address = bitcoin::Address::from_script(
        &bitcoin::Script::from_bytes(pkscript.iter().as_slice()),
        get_bitcoin_network(),
    );

    let Ok(address) = address else {
        return precompile_error(interpreter_result, "Failed to decode address");
    };

    let address = address;
    let message = message.iter().as_slice();
    let signature = signature.iter().as_slice();
    let signature = Witness::consensus_decode(&mut signature.iter().as_slice());

    let Ok(signature) = signature else {
        return precompile_error(interpreter_result, "Failed to decode signature");
    };

    let use_v0_0_11 = use_bip322_v0_0_11(call.block_height.saturating_to::<u64>());
    if !verify_simple(&address, message, signature, use_v0_0_11) {
        return precompile_error(interpreter_result, "Failed to verify signature");
    }

    return precompile_output(interpreter_result, verifyCall::abi_encode_returns(&true));
}

fn verify_simple(address: &Address, message: &[u8], signature: Witness, use_v0_0_11: bool) -> bool {
    if use_v0_0_11 {
        bip322::verify_simple(address, message, signature).is_ok()
    } else {
        bip322_legacy::verify_simple(address, message, signature).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use bitcoin::consensus::Encodable;
    use revm::primitives::U256;

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

        let signature = bip322::sign_simple(&address, &message, &[wif_private_key], None).unwrap();

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
            block_height: U256::ZERO,
            current_op_return_tx_id: [0u8; 32].into(),
            btc_tx_hexes_data: HashMap::new(),
            bitcoin_lookup: crate::engine::precompiles::BitcoinLookup::BlockExecution,
        });

        assert!(result.is_ok());

        let success = verifyCall::abi_decode_returns(&result.output).unwrap();
        assert!(success);
    }

    // BIP322 precompile inputs from mainnet block 913183
    // P2SH-P2WPKH proof whose ECDSA signature has a 69 byte encoding (68 byte DER)
    const SHORT_SIG_P2SH_P2WPKH_INPUT: &str = "de8f50a1000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000017a9142d6036e9df942d2308f7e09374402bfa2403ad9f8700000000000000000000000000000000000000000000000000000000000000000000000000000000b73078353637396662346564653830353165646262643838653031343264343731643332613963393938307c3078383035336164396637313365303936386364626537623130613736633361623063613964396136303565323233376437393730386466636339663830333935377c3078303836383b3078303836303b7c307830356239323335383964633133626233653435333b307832346130653233373234643462313463316261643b7c3078303339647c30783535000000000000000000000000000000000000000000000000000000000000000000000000000000006902453042022045d356273b452a56617861118bd71c634fa761dfa0970d4f85ab236a7fe11641021e1cbf9923cd0cda46f2e278ad67226f4d615f2423521a89181b3c1469909e01210232709fd2216ebf62211d7969097e77eb009a7e250d7ae1da559b73d61d0180ff0000000000000000000000000000000000000000000000";
    const P2TR_INPUT: &str = "de8f50a1000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000002251208b19f1f08b21387d6abf27cf2cfae7133d15a86b0673e7730b11acc0a20314b500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000b73078346261313566643531663561623063333132333338393364663663643038323833623538306130617c3078363964343366613437373762616665373661636462306362663039313965643733666535613139343937646333343932663330643065373636396261363532317c3078303838633b3078303836383b7c307832396565633163323532633533303337633030303b307837636162643563643934613733336234636262383b7c3078303336317c30783535000000000000000000000000000000000000000000000000000000000000000000000000000000004201408384d5e499930cba316cfbabd52c03cdf55ba35625f5d4bef41ba6a2a697eca5cea37059a51841c49f918532e6f84da39365c5be1e4a08a79010741080fb95bb000000000000000000000000000000000000000000000000000000000000";
    const P2WPKH_INPUT: &str = "de8f50a1000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000001c000000000000000000000000000000000000000000000000000000000000000160014510f8f71a383c633decc10a001e0fe545eb0ef8b0000000000000000000000000000000000000000000000000000000000000000000000000000000000ef3078623032383665343666396366303262323237666230396439323863333339633062346431393261337c3078613864626435343834396134306630326336376563626162363230333061366230343539353332383330316163653563636239353866386534636232373663337c3078303838613b3078303837383b3078303837643b3078303735393b7c307830313238373766663535643463393532653234303b30783835376536383231383134393434313262393b307830316561633337663966643161626565363030303b30786133373964613763346461356661616230373b7c3078303365347c307835350000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006b0247304402205d7c74c613bf8f23209574f2675640406b2c0f4f5dcbbe8a8d83f3121e52680402201de840e917f711de4b3249b8eb1808f4f1bee67d2b8e4e864c9a5d5e9d7944970121039cd21fbbfb2ffd7ce1f1e6f1647e36ae84a481ef378a2122d95ee45a11e83709000000000000000000000000000000000000000000";

    fn verify_mainnet_input(input: &str, use_v0_0_11: bool) -> bool {
        let inputs = verifyCall::abi_decode(&hex::decode(input).unwrap()).unwrap();
        let address = bitcoin::Address::from_script(
            bitcoin::Script::from_bytes(&inputs.pkscript),
            bitcoin::Network::Bitcoin,
        )
        .unwrap();
        let signature = Witness::consensus_decode(&mut inputs.signature.as_ref()).unwrap();
        verify_simple(&address, &inputs.message, signature, use_v0_0_11)
    }

    #[test]
    fn test_short_signature_rejected_before_v0_0_11() {
        assert!(!verify_mainnet_input(SHORT_SIG_P2SH_P2WPKH_INPUT, false));
        assert!(verify_mainnet_input(SHORT_SIG_P2SH_P2WPKH_INPUT, true));
    }

    #[test]
    fn test_valid_signatures_accepted_by_both_rule_sets() {
        for input in [P2TR_INPUT, P2WPKH_INPUT] {
            assert!(verify_mainnet_input(input, false));
            assert!(verify_mainnet_input(input, true));
        }
    }

    #[test]
    fn test_legacy_rejects_empty_p2tr_witness() {
        let inputs = verifyCall::abi_decode(&hex::decode(P2TR_INPUT).unwrap()).unwrap();
        let address = bitcoin::Address::from_script(
            bitcoin::Script::from_bytes(&inputs.pkscript),
            bitcoin::Network::Bitcoin,
        )
        .unwrap();
        assert!(!verify_simple(
            &address,
            &inputs.message,
            Witness::new(),
            false
        ));
    }
}
