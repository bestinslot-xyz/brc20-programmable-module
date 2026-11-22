use alloy::primitives::{Bytes, U256};
use alloy::sol_types::{sol, SolCall};
use revm::interpreter::{Gas, InstructionResult, InterpreterResult};

use crate::engine::precompiles::btc_utils::{
    get_block_height, get_transaction_and_block_hash_with_overrides, BitcoinDataUnavailable,
};
use crate::engine::precompiles::{precompile_error, precompile_output, use_gas, PrecompileCall};
use crate::global::{GAS_PER_BITCOIN_RPC_CALL, GAS_PER_WITNESS_TX_BYTE};

/*
    Signature for the getTxWitness function in the BTC_TxWitness precompile
    Returns the witness stack of one input of a confirmed transaction, exactly as serialized,
    including the annex and control block when present

    # Returns (block_height, vin_count, stack) in a tuple
    # Errors - Returns an error if the transaction is missing, unconfirmed, above the current
    # EVM height, or if vin is out of bounds
*/
sol! {
    function getTxWitness(bytes32 txid, uint256 vin) returns (uint256 block_height, uint256 vin_count, bytes[] stack);
}

const TX_NOT_AVAILABLE: &str = "Transaction is not confirmed at this height";

/// `Err` means a Bitcoin lookup failed during a public call. See
/// [`BitcoinLookup`](crate::engine::precompiles::BitcoinLookup).
pub fn btc_tx_witness_precompile(
    call: &PrecompileCall,
) -> Result<InterpreterResult, BitcoinDataUnavailable> {
    let mut interpreter_result = InterpreterResult::new(
        InstructionResult::Stop,
        Bytes::new(),
        Gas::new(call.gas_limit),
    );

    if !use_gas(&mut interpreter_result, GAS_PER_BITCOIN_RPC_CALL) {
        return Ok(interpreter_result);
    }

    let Ok(inputs) = getTxWitnessCall::abi_decode(&call.bytes) else {
        return Ok(precompile_error(
            interpreter_result,
            "Failed to decode parameters",
        ));
    };

    // A transaction that is unknown, in this node's mempool, or confirmed above the current height
    // must look the same to a contract. Mempools differ between nodes, so a different error or gas
    // charge here would let block execution diverge.
    let Some((tx, Some(block_hash))) = get_transaction_and_block_hash_with_overrides(
        &inputs.txid,
        &call.btc_tx_hexes_data,
        call.bitcoin_lookup,
    )?
    else {
        return Ok(precompile_error(interpreter_result, TX_NOT_AVAILABLE));
    };

    let block_height = get_block_height(&block_hash, call.bitcoin_lookup)?;

    if U256::from(block_height) > call.block_height {
        return Ok(precompile_error(interpreter_result, TX_NOT_AVAILABLE));
    }

    // The node serves and this process decodes the whole transaction whichever input is asked
    // for, so the charge follows the full serialized size and not the returned stack.
    if !use_gas(
        &mut interpreter_result,
        (tx.total_size() as u64).saturating_mul(GAS_PER_WITNESS_TX_BYTE),
    ) {
        return Ok(interpreter_result);
    }

    let Some(txin) = usize::try_from(inputs.vin)
        .ok()
        .and_then(|vin| tx.input.get(vin))
    else {
        return Ok(precompile_error(
            interpreter_result,
            "Vin index out of bounds",
        ));
    };

    let stack: Vec<Bytes> = txin
        .witness
        .iter()
        .map(|item| Bytes::copy_from_slice(item))
        .collect();

    let bytes = getTxWitnessCall::abi_encode_returns_tuple(&(
        U256::from(block_height),
        U256::from(tx.input.len()),
        stack,
    ));

    Ok(precompile_output(interpreter_result, bytes))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use alloy::primitives::{FixedBytes, B256};
    use bitcoin::absolute::LockTime;
    use bitcoin::consensus::serialize;
    use bitcoin::hashes::Hash;
    use bitcoin::transaction::Version;
    use bitcoin::{Amount, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid, Witness};

    use super::*;
    use crate::engine::precompiles::BitcoinLookup;

    fn tx_with_witnesses(witnesses: Vec<Vec<Vec<u8>>>) -> Transaction {
        Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: witnesses
                .into_iter()
                .enumerate()
                .map(|(i, items)| TxIn {
                    previous_output: OutPoint::new(Txid::from_byte_array([i as u8 + 1; 32]), 0),
                    script_sig: ScriptBuf::new(),
                    sequence: Sequence::MAX,
                    witness: Witness::from_slice(&items),
                })
                .collect(),
            output: vec![TxOut {
                value: Amount::from_sat(1_000),
                script_pubkey: ScriptBuf::from_bytes(vec![0x51]),
            }],
        }
    }

    fn call_for(tx: &Transaction, vin: U256, gas_limit: u64) -> PrecompileCall {
        let txid: B256 = tx.compute_txid().to_string().parse().unwrap();
        let mut overrides = HashMap::new();
        overrides.insert(txid, Bytes::from(serialize(tx)));
        PrecompileCall {
            bytes: getTxWitnessCall::new((txid, vin)).abi_encode().into(),
            gas_limit,
            block_height: U256::from(10u64),
            current_op_return_tx_id: [0u8; 32].into(),
            btc_tx_hexes_data: overrides,
            bitcoin_lookup: BitcoinLookup::BlockExecution,
        }
    }

    #[test]
    fn test_get_tx_witness_encode_params() {
        let txid = FixedBytes::<32>::from([0x11; 32]);
        let data = getTxWitnessCall::new((txid, U256::from(2u64))).abi_encode();
        assert_eq!(data.len(), 4 + 32 + 32);
        assert_eq!(&data[4..36], txid.as_slice());
        assert_eq!(data[67], 2);
    }

    #[test]
    fn test_returns_requested_input_stack() {
        let tx = tx_with_witnesses(vec![
            vec![vec![0xaa; 64]],
            vec![vec![1, 2, 3], vec![], vec![0xc0; 33]],
        ]);
        let result =
            btc_tx_witness_precompile(&call_for(&tx, U256::from(1u64), 10_000_000)).unwrap();
        assert!(result.is_ok());

        let returns = getTxWitnessCall::abi_decode_returns(&result.output).unwrap();
        assert_eq!(returns.block_height, U256::ZERO);
        assert_eq!(returns.vin_count, U256::from(2u64));
        assert_eq!(
            returns.stack,
            vec![
                Bytes::from(vec![1, 2, 3]),
                Bytes::new(),
                Bytes::from(vec![0xc0; 33])
            ]
        );
    }

    #[test]
    fn test_annex_is_returned_unmodified() {
        let tx = tx_with_witnesses(vec![vec![vec![0x51], vec![0xc0; 33], vec![0x50, 0x01]]]);
        let result = btc_tx_witness_precompile(&call_for(&tx, U256::ZERO, 10_000_000)).unwrap();
        let returns = getTxWitnessCall::abi_decode_returns(&result.output).unwrap();
        assert_eq!(returns.stack.len(), 3);
        assert_eq!(returns.stack[2], Bytes::from(vec![0x50, 0x01]));
    }

    #[test]
    fn test_vin_out_of_bounds_fails() {
        let tx = tx_with_witnesses(vec![vec![vec![1]]]);
        for vin in [U256::from(1u64), U256::from(1u64) << 64, U256::MAX] {
            let result = btc_tx_witness_precompile(&call_for(&tx, vin, 10_000_000)).unwrap();
            assert!(result.is_error());
            assert_eq!(
                result.output,
                Bytes::from_static(b"Vin index out of bounds")
            );
        }
    }

    #[test]
    fn test_gas_is_base_plus_full_transaction_size() {
        // Asking for the small input of a large transaction pays for the large transaction
        let tx = tx_with_witnesses(vec![vec![vec![1]], vec![vec![0xbb; 10_000]]]);
        let result = btc_tx_witness_precompile(&call_for(&tx, U256::ZERO, 10_000_000)).unwrap();
        assert!(result.is_ok());
        assert_eq!(
            result.gas.total_gas_spent(),
            GAS_PER_BITCOIN_RPC_CALL + tx.total_size() as u64 * GAS_PER_WITNESS_TX_BYTE
        );
    }

    #[test]
    fn test_out_of_gas_on_size_charge() {
        let tx = tx_with_witnesses(vec![vec![vec![0xbb; 10_000]]]);
        let result =
            btc_tx_witness_precompile(&call_for(&tx, U256::ZERO, GAS_PER_BITCOIN_RPC_CALL + 1))
                .unwrap();
        assert_eq!(result.result, InstructionResult::OutOfGas);
        assert!(result.output.is_empty());
    }

    #[test]
    fn test_out_of_gas_on_base_charge() {
        let tx = tx_with_witnesses(vec![vec![vec![1]]]);
        let result =
            btc_tx_witness_precompile(&call_for(&tx, U256::ZERO, GAS_PER_BITCOIN_RPC_CALL - 1))
                .unwrap();
        assert_eq!(result.result, InstructionResult::OutOfGas);
    }

    #[test]
    fn test_bad_calldata_fails() {
        let tx = tx_with_witnesses(vec![vec![vec![1]]]);
        let mut call = call_for(&tx, U256::ZERO, 10_000_000);
        call.bytes = Bytes::from_static(&[1, 2, 3]);
        let result = btc_tx_witness_precompile(&call).unwrap();
        assert!(result.is_error());
    }
}
