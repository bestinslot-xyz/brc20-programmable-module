use alloy_sol_types::{sol, SolCall};
use bitcoin::address::NetworkUnchecked;
use bitcoin::key::UntweakedPublicKey;
use bitcoin::taproot::TaprootBuilder;
use bitcoin::{opcodes, secp256k1, Address, ScriptBuf};
use revm::interpreter::{Gas, InstructionResult, InterpreterResult};
use revm::primitives::Bytes;

use crate::evm::precompiles::btc_utils::{BITCOIN_HRP, BITCOIN_NETWORK};
use crate::evm::precompiles::{precompile_error, precompile_output, use_gas};

sol! {
    function getLockedPkscript(string, uint256) returns (string);
}

pub fn get_locked_pkscript_precompile(bytes: &Bytes, gas_limit: u64) -> InterpreterResult {
    let mut interpreter_result =
        InterpreterResult::new(InstructionResult::Stop, Bytes::new(), Gas::new(gas_limit));

    if !use_gas(&mut interpreter_result, 20000) {
        return interpreter_result;
    }

    let result = getLockedPkscriptCall::abi_decode(&bytes, false);

    if result.is_err() {
        // Invalid params
        return precompile_error(interpreter_result);
    }

    let returns = result.unwrap();

    let pkscript = returns._0;
    let lock_block_count = returns._1.as_limbs()[0];

    if lock_block_count == 0 || lock_block_count > 65535 {
        // Invalid lock block count
        return precompile_error(interpreter_result);
    }

    let result = get_p2tr_lock_addr(&pkscript, lock_block_count);

    let bytes = getLockedPkscriptCall::abi_encode_returns(&(result,));

    return precompile_output(interpreter_result, bytes);
}

fn get_p2tr_lock_addr(pkscript: &String, lock_block_count: u64) -> String {
    let secp256k1 = secp256k1::Secp256k1::new();
    let lock_address = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0"; // Unspendable address

    let lock_script = TaprootBuilder::new()
        .add_leaf(0, build_lock_script(pkscript, lock_block_count))
        .unwrap()
        .finalize(
            &secp256k1,
            UntweakedPublicKey::from_slice(&hex::decode(lock_address).unwrap()).unwrap(),
        )
        .unwrap();

    Address::p2tr_tweaked(lock_script.output_key(), *BITCOIN_HRP).to_string()
}

fn build_lock_script(pkscript: &String, lock_block_count: u64) -> bitcoin::ScriptBuf {
    let pkscript = pkscript
        .parse::<Address<NetworkUnchecked>>()
        .unwrap()
        .require_network(*BITCOIN_NETWORK)
        .unwrap();
    let pubkey = pkscript.script_pubkey();
    let mut script = ScriptBuf::new();
    if lock_block_count <= 16 {
        script
            .push_opcode((opcodes::all::OP_PUSHNUM_1.to_u8() - 1 + lock_block_count as u8).into());
    } else {
        let mut lock_block_count_hex = lock_block_count.to_be_bytes().to_vec();
        while lock_block_count_hex.len() > 1 && lock_block_count_hex[0] == 0 {
            lock_block_count_hex.remove(0);
        }
        if lock_block_count_hex[0] >= 0x80 {
            lock_block_count_hex.insert(0, 0x00);
        }
        match lock_block_count_hex.len() {
            1 => {
                script.push_slice(&[lock_block_count_hex[0]]);
            }
            2 => {
                script.push_slice(&[lock_block_count_hex[1], lock_block_count_hex[0]]);
            }
            3 => {
                script.push_slice(&[
                    lock_block_count_hex[2],
                    lock_block_count_hex[1],
                    lock_block_count_hex[0],
                ]);
            }
            _ => {
                script.push_slice(&[
                    lock_block_count_hex[3],
                    lock_block_count_hex[2],
                    lock_block_count_hex[1],
                    lock_block_count_hex[0],
                ]);
            }
        }
    }

    script.push_opcode(opcodes::all::OP_CSV);
    script.push_opcode(opcodes::all::OP_DROP);
    let pubkey_bytes: [u8; 32] = pubkey.to_bytes().as_slice()[2..].try_into().unwrap();
    script.push_slice(pubkey_bytes);
    script.push_opcode(opcodes::all::OP_CHECKSIG);

    script
}

#[cfg(test)]
mod tests {
    use alloy_primitives::U256;

    use super::*;

    #[test]
    fn test_get_locked_pkscript_six_blocks() {
        let bytes = getLockedPkscriptCall::new((
            "tb1plnw9577kddxn4ry37xsul99d04tp7w3sf0cclt6k0zc7u3l8swms7vfp48".to_string(),
            U256::from(6u8),
        ))
        .abi_encode();
        let result = get_locked_pkscript_precompile(&bytes.into(), 100000);
        let result = getLockedPkscriptCall::abi_decode_returns(&result.output, false).unwrap();
        assert_eq!(
            result._0,
            "tb1ppnn9pkm5qrdx99lypxxka3zhs322qse4x88y39r8z2vfjhk7ex4sfu7cgf"
        )
    }

    #[test]
    fn test_get_locked_pkscript_year_lock() {
        let bytes = getLockedPkscriptCall::new((
            "tb1plnw9577kddxn4ry37xsul99d04tp7w3sf0cclt6k0zc7u3l8swms7vfp48".to_string(),
            U256::from(52560u32),
        ))
        .abi_encode();
        let result = get_locked_pkscript_precompile(&bytes.into(), 100000);
        let result = getLockedPkscriptCall::abi_decode_returns(&result.output, false).unwrap();
        assert_eq!(
            result._0,
            "tb1p9p7v3afn2zptdq4cjvl7376p63vhdy7y53uayftmamuh8mp4ynmsvaeu4e"
        )
    }

    #[test]
    fn test_get_locked_pkscript_max_lock() {
        let bytes = getLockedPkscriptCall::new((
            "tb1plnw9577kddxn4ry37xsul99d04tp7w3sf0cclt6k0zc7u3l8swms7vfp48".to_string(),
            U256::from(65535u32),
        ))
        .abi_encode();
        let result = get_locked_pkscript_precompile(&bytes.into(), 100000);
        let result = getLockedPkscriptCall::abi_decode_returns(&result.output, false).unwrap();
        assert_eq!(
            result._0,
            "tb1pp7kk3e79nhvt5pyjhqfwgaxq8zfm5vze4duy2f7xds4mfv0z24ssvnkfzw"
        )
    }

    #[test]
    fn test_get_locked_pkscript_zero_lock() {
        let bytes = getLockedPkscriptCall::new((
            "tb1plnw9577kddxn4ry37xsul99d04tp7w3sf0cclt6k0zc7u3l8swms7vfp48".to_string(),
            U256::from(0u32),
        ))
        .abi_encode();
        let result = get_locked_pkscript_precompile(&bytes.into(), 100000);
        assert!(result.is_error());
    }

    #[test]
    fn test_get_locked_pkscript_max_plus_one_lock() {
        let bytes = getLockedPkscriptCall::new((
            "tb1plnw9577kddxn4ry37xsul99d04tp7w3sf0cclt6k0zc7u3l8swms7vfp48".to_string(),
            U256::from(65536u32),
        ))
        .abi_encode();
        let result = get_locked_pkscript_precompile(&bytes.into(), 100000);
        assert!(result.is_error());
    }
}
