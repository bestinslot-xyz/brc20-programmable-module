use alloy_primitives::{Bytes, U256};
use alloy_sol_types::{sol, SolCall};
use bitcoin::key::UntweakedPublicKey;
use bitcoin::script::PushBytesBuf;
use bitcoin::taproot::TaprootBuilder;
use bitcoin::{opcodes, secp256k1, ScriptBuf};
use revm::interpreter::{Gas, InstructionResult, InterpreterResult};

use crate::evm::precompiles::{
    precompile_error, precompile_output, use_gas, PrecompileCall, BITCOIN_NETWORK,
};

sol! {
    function getLockedPkscript(bytes pkscript, uint256 lock_block_count) returns (bytes locked_pkscript);
}

pub fn get_locked_pkscript_precompile(call: &PrecompileCall) -> InterpreterResult {
    let mut interpreter_result = InterpreterResult::new(
        InstructionResult::Stop,
        Bytes::new(),
        Gas::new(call.gas_limit),
    );

    if !use_gas(&mut interpreter_result, 20000) {
        return interpreter_result;
    }

    let Ok(inputs) = getLockedPkscriptCall::abi_decode(&call.bytes) else {
        return precompile_error(interpreter_result);
    };

    if inputs.lock_block_count == U256::ZERO
        || inputs.lock_block_count > U256::from_limbs([65535u64, 0u64, 0u64, 0u64])
    {
        // Invalid lock block count
        return precompile_error(interpreter_result);
    }

    let lock_block_count = inputs.lock_block_count.as_limbs()[0];

    let Ok(result) = get_p2tr_lock_addr(&inputs.pkscript, lock_block_count) else {
        // Failed to get lock address
        return precompile_error(interpreter_result);
    };

    let bytes = getLockedPkscriptCall::abi_encode_returns(&result);

    return precompile_output(interpreter_result, bytes);
}

fn get_p2tr_lock_addr(pkscript: &Bytes, lock_block_count: u64) -> Result<Bytes, &'static str> {
    let secp256k1 = secp256k1::Secp256k1::new();
    let lock_address = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0"; // Unspendable address

    let lock_script_leaf =
        build_lock_script(pkscript, lock_block_count).map_err(|_| "Failed to build lock script")?;

    let lock_script = TaprootBuilder::new()
        .add_leaf(0, lock_script_leaf)
        .map_err(|_| "Failed to add leaf")?
        .finalize(
            &secp256k1,
            UntweakedPublicKey::from_slice(
                &hex::decode(lock_address).map_err(|_| "Invalid lock address")?,
            )
            .map_err(|_| "Failed to create untweaked public key")?,
        )
        .map_err(|_| "Failed to finalize taproot")?;

    let address = bitcoin::Address::p2tr_tweaked(lock_script.output_key(), *BITCOIN_NETWORK);

    Ok(Bytes::from(address.script_pubkey().into_bytes()))
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
    push_bytes
        .extend_from_slice(pkscript.slice(2..).iter().as_slice())
        .map_err(|_| "Failed to push bytes")?;

    script.push_instruction(bitcoin::script::Instruction::PushBytes(
        &push_bytes.as_push_bytes(),
    ));

    script.push_opcode(opcodes::all::OP_CHECKSIG);

    Ok(script)
}

#[cfg(test)]
mod tests {
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
        let result = get_locked_pkscript_precompile(&PrecompileCall {
            bytes: bytes.into(),
            gas_limit: 100000,
            block_height: 0,
        });
        let result = getLockedPkscriptCall::abi_decode_returns(&result.output).unwrap();
        assert_eq!(
            hex::encode(result.iter().as_slice()),
            "51206ec7460e24bdaeba7384e2d5ff54a4645e0b53854594d225c52a4195eba194ca"
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
        let result = get_locked_pkscript_precompile(&PrecompileCall {
            bytes: bytes.into(),
            gas_limit: 100000,
            block_height: 0,
        });
        let result = getLockedPkscriptCall::abi_decode_returns(&result.output).unwrap();
        assert_eq!(
            hex::encode(result.iter().as_slice()),
            "512015f5761f81118dddccfe8a25ed99a2b8c6f3a0782efd76f0871b45c6737b1f7e"
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
        let result = get_locked_pkscript_precompile(&PrecompileCall {
            bytes: bytes.into(),
            gas_limit: 100000,
            block_height: 0,
        });
        let result = getLockedPkscriptCall::abi_decode_returns(&result.output).unwrap();
        assert_eq!(
            hex::encode(result.iter().as_slice()),
            "5120397a9ad3d17d8601ff2f139be6b9b7b5d0e0daac8565c27f617f8eaf7719ab9d"
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
        let result = get_locked_pkscript_precompile(&PrecompileCall {
            bytes: bytes.into(),
            gas_limit: 100000,
            block_height: 0,
        });
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
        let result = get_locked_pkscript_precompile(&PrecompileCall {
            bytes: bytes.into(),
            gas_limit: 100000,
            block_height: 0,
        });
        assert!(result.is_error());
    }
}
