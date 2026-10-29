//! A commit or a reorg is persisted whole or not at all: a failure at any point of a commit
//! leaves the store at the previous block or at the new one, and re-executing a lost block
//! gives the same traces and receipts.

use std::path::Path;

use alloy::primitives::{Address, Bytes, B256, U256};
use revm::primitives::TxKind;
use tempfile::TempDir;

use crate::db::store::test_hooks::{self, StoreDump};
use crate::db::store::{self, FailPoint};
use crate::db::types::TxReceiptED;
use crate::db::Brc20ProgDatabase;
use crate::engine::{BRC20ProgEngine, TxInfo};
use crate::global::CONFIG;

const TIMESTAMP: u64 = 1_700_000_000;
const SENDER: [u8; 20] = [0x11; 20];

/// Init code that stores 42 in slot 0, emits a log, and deploys a runtime that increments
/// slot 0 on every call.
fn init_code() -> Bytes {
    // SSTORE(0, 42); LOG0(0, 1); CODECOPY(0, 0x16, 10); RETURN(0, 10)
    // runtime: SSTORE(0, SLOAD(0) + 1); STOP
    hex::decode(concat!(
        "602a600055",
        "60016000a0",
        "600a60166000",
        "39",
        "600a6000f3",
        "60005460010160005500"
    ))
    .unwrap()
    .into()
}

/// Column families whose values hold wall-clock processing time.
const WALL_CLOCK_COLUMN_FAMILIES: [&str; 2] =
    ["block_number_to_block", "block_number_to_raw_block"];

fn open_engine(path: &Path) -> BRC20ProgEngine {
    BRC20ProgEngine::new(Brc20ProgDatabase::new(path).unwrap())
}

fn dump(path: &Path) -> StoreDump {
    test_hooks::dump_store(&store::open(path).unwrap())
}

fn without_wall_clock(mut dump: StoreDump) -> StoreDump {
    for name in WALL_CLOCK_COLUMN_FAMILIES {
        dump.remove(name);
    }
    dump
}

fn add_tx(engine: &BRC20ProgEngine, block: u64, idx: u64, to: TxKind, label: &str) -> TxReceiptED {
    let data = match to {
        TxKind::Create => init_code(),
        TxKind::Call(_) => Bytes::new(),
    };
    engine
        .add_tx_to_block(
            TIMESTAMP + block,
            &TxInfo::from_inscription(Address::from(SENDER), to, data),
            idx,
            block,
            B256::ZERO,
            format!("{}i{}", label, block * 100 + idx),
            1_000,
            B256::ZERO,
        )
        .unwrap()
}

/// Genesis, then block 1 deploys the contract and calls it. Returns the contract address.
fn run_first_blocks(engine: &BRC20ProgEngine) -> Address {
    engine.mine_blocks(1, TIMESTAMP).unwrap();
    engine.commit_to_db().unwrap();

    let receipt = add_tx(engine, 1, 0, TxKind::Create, "deploy");
    let contract = receipt.contract_address.unwrap().address;
    add_tx(engine, 1, 1, TxKind::Call(contract), "call");
    engine
        .finalise_block(TIMESTAMP + 1, 1, B256::ZERO, 2)
        .unwrap();
    engine.commit_to_db().unwrap();
    contract
}

/// Block 2 calls the contract twice, without committing.
fn execute_block_two(engine: &BRC20ProgEngine, contract: Address) -> Vec<TxReceiptED> {
    let receipts = vec![
        add_tx(engine, 2, 0, TxKind::Call(contract), "call"),
        add_tx(engine, 2, 1, TxKind::Call(contract), "call"),
    ];
    engine
        .finalise_block(TIMESTAMP + 2, 2, B256::ZERO, 2)
        .unwrap();
    receipts
}

/// What a block must look like after it is persisted.
#[derive(Debug, PartialEq)]
struct BlockOutcome {
    trace_hash: Option<String>,
    raw_receipts: Option<Vec<String>>,
    receipts: Vec<Option<TxReceiptED>>,
    block_hash: Option<B256>,
    slot_zero: U256,
}

fn outcome(engine: &BRC20ProgEngine, contract: Address, receipts: &[TxReceiptED]) -> BlockOutcome {
    BlockOutcome {
        trace_hash: engine.get_block_trace_hash(2).unwrap(),
        raw_receipts: engine.get_raw_receipts(2).unwrap(),
        receipts: receipts
            .iter()
            .map(|r| {
                engine
                    .get_transaction_receipt(r.transaction_hash.bytes)
                    .unwrap()
            })
            .collect(),
        block_hash: engine
            .get_block_by_number(2, false)
            .unwrap()
            .map(|b| b.hash.bytes),
        slot_zero: engine.get_storage_at(contract, U256::ZERO).unwrap(),
    }
}

/// An uninterrupted run: the outcome of block 2 and the store contents after it.
fn reference() -> (BlockOutcome, StoreDump) {
    let dir = TempDir::new().unwrap();
    let (result, receipts_len) = {
        let engine = open_engine(dir.path());
        let contract = run_first_blocks(&engine);
        let receipts = execute_block_two(&engine, contract);
        engine.commit_to_db().unwrap();
        (outcome(&engine, contract, &receipts), receipts.len())
    };
    assert_eq!(receipts_len, 2);
    assert!(result.trace_hash.is_some());
    assert_eq!(result.slot_zero, U256::from(45));
    (result, without_wall_clock(dump(dir.path())))
}

fn record_traces() {
    CONFIG.write_fn_unchecked(|config| config.evm_record_traces = true);
}

#[test]
fn test_failure_before_the_batch_write_leaves_the_previous_block() {
    record_traces();
    let (expected, expected_dump) = reference();

    let dir = TempDir::new().unwrap();
    let contract = run_first_blocks(&open_engine(dir.path()));
    let before_block_two = dump(dir.path());

    {
        let engine = open_engine(dir.path());
        execute_block_two(&engine, contract);
        test_hooks::arm(FailPoint::CommitBeforeWrite);
        assert!(engine.commit_to_db().is_err());
        test_hooks::disarm();
        // The process dies here: the engine is dropped with block 2 only in memory.
    }

    // All or nothing: not one key of block 2 reached any column family.
    assert_eq!(dump(dir.path()), before_block_two);

    let engine = open_engine(dir.path());
    assert_eq!(engine.get_latest_block_height().unwrap(), 1);
    assert!(engine.get_block_by_number(2, false).unwrap().is_none());
    assert_eq!(
        engine.get_storage_at(contract, U256::ZERO).unwrap(),
        U256::from(43)
    );

    // Re-executing block 2 gives the same traces and receipts as the uninterrupted run.
    let receipts = execute_block_two(&engine, contract);
    engine.commit_to_db().unwrap();
    assert_eq!(outcome(&engine, contract, &receipts), expected);
    drop(engine);
    assert_eq!(without_wall_clock(dump(dir.path())), expected_dump);
}

#[test]
fn test_failure_after_the_batch_write_keeps_the_new_block() {
    record_traces();
    let (expected, expected_dump) = reference();

    let dir = TempDir::new().unwrap();
    let (contract, receipts) = {
        let engine = open_engine(dir.path());
        let contract = run_first_blocks(&engine);
        let receipts = execute_block_two(&engine, contract);
        test_hooks::arm(FailPoint::CommitAfterWrite);
        assert!(engine.commit_to_db().is_err());
        test_hooks::disarm();
        (contract, receipts)
    };

    let engine = open_engine(dir.path());
    assert_eq!(engine.get_latest_block_height().unwrap(), 2);
    assert_eq!(outcome(&engine, contract, &receipts), expected);
    drop(engine);
    assert_eq!(without_wall_clock(dump(dir.path())), expected_dump);
}

#[test]
fn test_reorg_is_written_in_one_batch() {
    let dir = TempDir::new().unwrap();
    let contract = {
        let engine = open_engine(dir.path());
        let contract = run_first_blocks(&engine);
        execute_block_two(&engine, contract);
        engine.commit_to_db().unwrap();
        contract
    };
    let before_reorg = dump(dir.path());

    {
        let engine = open_engine(dir.path());
        test_hooks::arm(FailPoint::CommitBeforeWrite);
        assert!(engine.reorg(1).is_err());
        test_hooks::disarm();
    }
    assert_eq!(dump(dir.path()), before_reorg);

    {
        let engine = open_engine(dir.path());
        assert_eq!(engine.get_latest_block_height().unwrap(), 2);
        test_hooks::arm(FailPoint::CommitAfterWrite);
        assert!(engine.reorg(1).is_err());
        test_hooks::disarm();
    }

    let engine = open_engine(dir.path());
    assert_eq!(engine.get_latest_block_height().unwrap(), 1);
    assert!(engine.get_block_by_number(2, false).unwrap().is_none());
    assert_eq!(
        engine.get_storage_at(contract, U256::ZERO).unwrap(),
        U256::from(43)
    );
}
