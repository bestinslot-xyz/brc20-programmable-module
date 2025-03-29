use alloy_primitives::U256;
use alloy_sol_types::{sol, SolCall};
use bitcoin::key::UntweakedPublicKey;
use bitcoin::script::PushBytesBuf;
use bitcoin::taproot::TaprootBuilder;
use bitcoin::{opcodes, secp256k1, ScriptBuf};
use revm::interpreter::{Gas, InstructionResult, InterpreterResult};
use revm::primitives::Bytes;

use crate::evm::precompiles::{precompile_error, precompile_output, use_gas};

sol! {
    function getLockedPkscript(bytes pkscript, uint256 lock_block_count) returns (bytes locked_pkscript);
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

    if returns.lock_block_count == U256::ZERO
        || returns.lock_block_count > U256::from_limbs([65535u64, 0u64, 0u64, 0u64])
    {
        // Invalid lock block count
        return precompile_error(interpreter_result);
    }

    let lock_block_count = returns.lock_block_count.as_limbs()[0];

    let result = get_p2tr_lock_addr(&returns.pkscript, lock_block_count);

    if result.is_err() {
        // Invalid pkscript
        return precompile_error(interpreter_result);
    }

    let bytes = getLockedPkscriptCall::abi_encode_returns(&(result.unwrap(),));

    return precompile_output(interpreter_result, bytes);
}

fn get_p2tr_lock_addr(pkscript: &Bytes, lock_block_count: u64) -> Result<Bytes, &'static str> {
    let secp256k1 = secp256k1::Secp256k1::new();
    let lock_address = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0"; // Unspendable address

    let lock_script_leaf = build_lock_script(pkscript, lock_block_count);
    if lock_script_leaf.is_err() {
        return Err(lock_script_leaf.unwrap_err());
    }
    let lock_script_leaf = lock_script_leaf.unwrap();

    let lock_script = TaprootBuilder::new()
        .add_leaf(0, lock_script_leaf)
        .unwrap()
        .finalize(
            &secp256k1,
            UntweakedPublicKey::from_slice(&hex::decode(lock_address).unwrap()).unwrap(),
        )
        .unwrap();

    Ok(lock_script.output_key().serialize().into())
}

fn build_lock_script(
    pkscript: &Bytes,
    lock_block_count: u64,
) -> Result<bitcoin::ScriptBuf, &'static str> {
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

    let mut push_bytes = PushBytesBuf::new();
    let result = push_bytes.extend_from_slice(pkscript.iter().as_slice());
    if result.is_err() {
        return Err("Invalid PkScript");
    }

    script.push_instruction(bitcoin::script::Instruction::PushBytes(
        &push_bytes.as_push_bytes(),
    ));

    script.push_opcode(opcodes::all::OP_CHECKSIG);

    Ok(script)
}

#[cfg(test)]
mod tests {
    use alloy_primitives::U256;

    use super::*;

    #[test]
    fn test_get_locked_pkscript_six_blocks() {
        let bytes = getLockedPkscriptCall::new((
            hex::decode("5120e0e224cd541454519b62047aa0891ea7b81a16598556aeb83a412a0b06a20aab")
                .unwrap()
                .into(),
            U256::from(6u8),
        ))
        .abi_encode();
        let result = get_locked_pkscript_precompile(&bytes.into(), 100000);
        let result = getLockedPkscriptCall::abi_decode_returns(&result.output, false).unwrap();
        assert_eq!(
            hex::encode(result.locked_pkscript),
            "e7b4a96c9beec8711f12c0d9956d6313a592c5abd8f8a90de8cf5b6d16e9e58d"
        )
    }

    #[test]
    fn test_get_locked_pkscript_year_lock() {
        let bytes = getLockedPkscriptCall::new((
            hex::decode("5120e0e224cd541454519b62047aa0891ea7b81a16598556aeb83a412a0b06a20aab")
                .unwrap()
                .into(),
            U256::from(52560u32),
        ))
        .abi_encode();
        let result = get_locked_pkscript_precompile(&bytes.into(), 100000);
        let result = getLockedPkscriptCall::abi_decode_returns(&result.output, false).unwrap();
        assert_eq!(
            hex::encode(result.locked_pkscript),
            "6b6f9e9324995ef82b3dea1ee288f59214145d64f88d1a76c3424e5539bb6c5f"
        )
    }

    #[test]
    fn test_get_locked_pkscript_max_lock() {
        let bytes = getLockedPkscriptCall::new((
            hex::decode("5120e0e224cd541454519b62047aa0891ea7b81a16598556aeb83a412a0b06a20aab")
                .unwrap()
                .into(),
            U256::from(65535u32),
        ))
        .abi_encode();
        let result = get_locked_pkscript_precompile(&bytes.into(), 100000);
        let result = getLockedPkscriptCall::abi_decode_returns(&result.output, false).unwrap();
        assert_eq!(
            hex::encode(result.locked_pkscript),
            "e9c89c9102b18073802e24b5e4c39736aa8c1634b646c3c854bb0d3455af0d7a"
        )
    }

    #[test]
    fn test_get_locked_pkscript_zero_lock() {
        let bytes = getLockedPkscriptCall::new((
            hex::decode("5120e0e224cd541454519b62047aa0891ea7b81a16598556aeb83a412a0b06a20aab")
                .unwrap()
                .into(),
            U256::from(0u32),
        ))
        .abi_encode();
        let result = get_locked_pkscript_precompile(&bytes.into(), 100000);
        assert!(result.is_error());
    }

    #[test]
    fn test_get_locked_pkscript_max_plus_one_lock() {
        let bytes = getLockedPkscriptCall::new((
            hex::decode("5120e0e224cd541454519b62047aa0891ea7b81a16598556aeb83a412a0b06a20aab")
                .unwrap()
                .into(),
            U256::from(65536u32),
        ))
        .abi_encode();
        let result = get_locked_pkscript_precompile(&bytes.into(), 100000);
        assert!(result.is_error());
    }
}
