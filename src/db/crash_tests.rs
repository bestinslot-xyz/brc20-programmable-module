//! Crash tests: a commit or reorg stops after each of its writes, as a killed process would,
//! and reopening the store rolls it back to the last fully written height.

use std::path::Path;

use alloy::primitives::{Address, Bytes, B256, U256};
use revm::primitives::TxKind;
use tempfile::TempDir;

use super::*;
use alloy_sol_types::SolCall;

use crate::brc20_controller::{
    decode_brc20_balance_result, load_brc20_balance_tx, load_brc20_burn_tx, load_brc20_deploy_tx,
    load_brc20_mint_tx,
};
use crate::db::types::{Decode, Encode};
use crate::db::WRITES_BEFORE_CRASH;
use crate::engine::{BRC20ProgEngine, TxInfo};

const TIMESTAMP: u64 = 1622547800;

type Dump = Vec<(String, Vec<(Vec<u8>, Vec<u8>)>)>;

fn dump_cached<K, V, C>(db: &Option<BlockCachedDatabase<K, V, C>>) -> Vec<(Vec<u8>, Vec<u8>)>
where
    K: Encode + Decode + Eq + std::hash::Hash + Clone,
    V: Encode + Decode + Eq + Clone,
    C: crate::db::cached_database::BlockHistoryCache<V> + Encode + Decode + Clone,
{
    let mut entries: Vec<_> = db
        .as_ref()
        .unwrap()
        .all()
        .unwrap()
        .into_iter()
        .map(|(key, value)| (key.encode_vec(), value.encode_vec()))
        .collect();
    entries.sort();
    entries
}

fn dump_blocks<V: Encode + Decode + Clone>(
    db: &Option<BlockDatabase<V>>,
) -> Vec<(Vec<u8>, Vec<u8>)> {
    let db = db.as_ref().unwrap();
    let last = db.last_key().unwrap().unwrap_or(0);
    (0..=last)
        .filter_map(|n| db.get(n).unwrap().map(|v| (n.encode_vec(), v.encode_vec())))
        .collect()
}

/// The receipts of the raw blocks. Their transactions encode the chain ID, which other tests
/// change while these run.
fn dump_raw_receipts(db: &Option<BlockDatabase<RawBlock>>) -> Vec<(Vec<u8>, Vec<u8>)> {
    dump_blocks(db)
        .into_iter()
        .map(|(key, value)| {
            let receipts = RawBlock::decode_vec(&value).unwrap().raw_receipts();
            (key, receipts.concat().into_bytes())
        })
        .collect()
}

/// The blocks with their indexing time zeroed, as it differs between runs
fn dump_block_responses(db: &Option<BlockDatabase<BlockResponseED>>) -> Vec<(Vec<u8>, Vec<u8>)> {
    dump_blocks(db)
        .into_iter()
        .map(|(key, value)| {
            let mut block = BlockResponseED::decode_vec(&value).unwrap();
            block.mine_timestamp = 0u64.into();
            (key, block.encode_vec())
        })
        .collect()
}

impl Brc20ProgDatabase {
    /// Every value a read can see, by database. Traces stay out with `with_traces` false,
    /// as a reorg keeps the traces of the blocks it removes.
    fn dump(&self, with_traces: bool) -> Dump {
        let mut dump = vec![
            ("account_memory", dump_cached(&self.db_account_memory)),
            ("code", dump_cached(&self.db_code)),
            ("account", dump_cached(&self.db_account)),
            (
                "number_and_index",
                dump_cached(&self.db_number_and_index_to_tx_hash),
            ),
            ("tx_receipt", dump_cached(&self.db_tx_receipt)),
            ("tx", dump_cached(&self.db_tx)),
            ("pending_txes", dump_cached(&self.db_pending_txes)),
            (
                "pending_ids",
                dump_cached(&self.db_pending_txes_op_return_tx_ids),
            ),
            (
                "inscription_id",
                dump_cached(&self.db_inscription_id_to_tx_hash),
            ),
            (
                "contract_address",
                dump_cached(&self.db_contract_address_to_inscription_id),
            ),
            ("hash_to_number", dump_cached(&self.db_block_hash_to_number)),
            (
                "block",
                dump_block_responses(&self.db_block_number_to_block),
            ),
            (
                "raw_receipts",
                dump_raw_receipts(&self.db_block_number_to_raw_block),
            ),
            ("number_to_hash", dump_blocks(&self.db_block_number_to_hash)),
        ];
        if with_traces {
            dump.push(("tx_trace", dump_cached(&self.db_tx_trace)));
        }
        dump.into_iter()
            .map(|(name, v)| (name.to_string(), v))
            .collect()
    }

    fn has_marker(&self) -> bool {
        self.db_global_values
            .as_ref()
            .unwrap()
            .get(ROLLBACK_TARGET_KEY.to_string())
            .unwrap()
            .is_some()
    }
}

fn open_engine(path: &Path) -> BRC20ProgEngine {
    CONFIG.write_fn_unchecked(|config| config.evm_record_traces = true);
    BRC20ProgEngine::new(Brc20ProgDatabase::new(path).unwrap())
}

fn holder(n: u64) -> Address {
    Address::from_word(B256::from(U256::from(n + 1)))
}

/// Executes block `n`: the controller deploy at 0, then mints, a burn and a contract deploy.
fn execute_block(engine: &BRC20ProgEngine, n: u64) {
    let ordi = Bytes::from("ordi".as_bytes().to_vec());
    let txes = if n == 0 {
        vec![load_brc20_deploy_tx()]
    } else {
        vec![
            load_brc20_mint_tx(ordi.clone(), holder(n % 3), U256::from(1000 + n)),
            load_brc20_mint_tx("sats".as_bytes().to_vec().into(), holder(n), U256::from(n)),
            load_brc20_burn_tx(ordi, holder(1), U256::from(7)),
            TxInfo::from_inscription(
                holder(n),
                TxKind::Create,
                vec![0x60, n as u8, 0x60, 0, 0xf3].into(),
            ),
        ]
    };
    for (idx, tx) in txes.iter().enumerate() {
        let id = format!("{}i{}", n, idx);
        let op_return = B256::from(U256::from(n * 100 + idx as u64));
        engine
            .add_tx_to_block(
                TIMESTAMP,
                tx,
                idx as u64,
                n,
                B256::ZERO,
                id,
                1000,
                op_return,
            )
            .unwrap();
    }
    engine
        .finalise_block(TIMESTAMP, n, B256::ZERO, txes.len() as u64)
        .unwrap();
}

/// Executes blocks `0..=to` into a new store, committing after `commits_at` and at `to`.
fn build(path: &Path, commit_at: u64, to: u64) {
    let engine = open_engine(path);
    for n in 0..=to {
        execute_block(&engine, n);
        if n == commit_at || n == to {
            engine.commit_to_db().unwrap();
        }
    }
}

fn assert_same(actual: &Dump, expected: &Dump) {
    for ((name, a), (_, e)) in actual.iter().zip(expected) {
        let extra: Vec<_> = a.iter().filter(|x| !e.contains(x)).collect();
        let missing: Vec<_> = e.iter().filter(|x| !a.contains(x)).collect();
        assert!(
            extra.is_empty() && missing.is_empty(),
            "{}: extra {:?}, missing {:?}",
            name,
            extra,
            missing
        );
    }
    assert_eq!(actual.len(), expected.len());
}

fn dump_at(path: &Path, with_traces: bool) -> Dump {
    Brc20ProgDatabase::new(path).unwrap().dump(with_traces)
}

/// Block outputs a re-execution must reproduce: hash, trace hash and raw receipts.
fn block_outputs(engine: &BRC20ProgEngine, n: u64) -> (B256, Option<String>, Option<Vec<String>>) {
    (
        engine
            .get_block_by_number(n, false)
            .unwrap()
            .unwrap()
            .hash
            .bytes,
        engine.get_block_trace_hash(n).unwrap(),
        engine.get_raw_receipts(n).unwrap(),
    )
}

/// Runs `interrupted` with the crash budget set to `writes`, and returns whether it crashed.
fn crash_after(writes: usize, interrupted: impl FnOnce() -> Result<(), Box<dyn Error>>) -> bool {
    WRITES_BEFORE_CRASH.set(Some(writes));
    let crashed = interrupted().is_err();
    WRITES_BEFORE_CRASH.set(None);
    crashed
}

/// Checks that the store at `path` reopened at `low` or `high`, matching the twin store, and
/// that re-executing up to `high` reproduces the uninterrupted run.
fn check_recovered(
    path: &Path,
    low: u64,
    high: u64,
    twin: &Dump,
    reference: &(Dump, Vec<(B256, Option<String>, Option<Vec<String>>)>),
) -> u64 {
    let db = Brc20ProgDatabase::new(path).unwrap();
    assert!(!db.has_marker());
    let height = db.get_latest_block_height().unwrap();
    if height == high {
        assert_same(&db.dump(true), &reference.0);
        return height;
    }
    assert_eq!(height, low);
    assert_same(&db.dump(false), twin);
    drop(db);

    let engine = open_engine(path);
    for n in low + 1..=high {
        execute_block(&engine, n);
    }
    engine.commit_to_db().unwrap();
    for n in low + 1..=high {
        assert_eq!(
            block_outputs(&engine, n),
            reference.1[(n - low - 1) as usize]
        );
    }
    drop(engine);
    assert_same(&dump_at(path, true), &reference.0);
    height
}

fn reference_run(low: u64, high: u64) -> (Dump, Vec<(B256, Option<String>, Option<Vec<String>>)>) {
    let dir = TempDir::new().unwrap();
    build(dir.path(), low, high);
    let engine = open_engine(dir.path());
    let outputs = (low + 1..=high)
        .map(|n| block_outputs(&engine, n))
        .collect();
    drop(engine);
    (dump_at(dir.path(), true), outputs)
}

#[test]
fn test_commit_crash_after_every_write_rolls_back() {
    // Blocks 0..=12 are committed, then blocks 13 and 14 go out in one commit. Holders first
    // minted more than MAX_REORG_HISTORY_SIZE blocks back keep their history as old ones.
    let (low, high) = (12, 14);
    let twin_dir = TempDir::new().unwrap();
    build(twin_dir.path(), low, low);
    let twin = dump_at(twin_dir.path(), false);
    let reference = reference_run(low, high);

    let mut heights = Vec::new();
    for writes in 1.. {
        let dir = TempDir::new().unwrap();
        let engine = open_engine(dir.path());
        for n in 0..=high {
            execute_block(&engine, n);
            if n == low {
                engine.commit_to_db().unwrap();
            }
        }
        if !crash_after(writes, || engine.commit_to_db()) {
            break;
        }
        drop(engine);
        heights.push(check_recovered(dir.path(), low, high, &twin, &reference));
    }

    // One crash per write: the marker, every instance batch, then the marker clear
    let last = heights.pop().unwrap();
    assert_eq!(last, high);
    assert!(heights.len() > 20);
    assert!(heights.iter().all(|&h| h == low));
}

#[test]
fn test_reorg_crash_after_every_write_rolls_back() {
    // The reorg undoes blocks 13..=15, whose new keys have a history old enough to delete
    let (low, high) = (12, 15);
    let twin_dir = TempDir::new().unwrap();
    build(twin_dir.path(), low, low);
    let twin = dump_at(twin_dir.path(), false);
    let reference = reference_run(low, high);

    let mut crashes = 0;
    for writes in 1.. {
        let dir = TempDir::new().unwrap();
        build(dir.path(), low + 1, high);
        let engine = open_engine(dir.path());
        if !crash_after(writes, || engine.reorg(low)) {
            break;
        }
        drop(engine);
        crashes += 1;
        // A crash after the marker clears leaves the finished reorg at `low`
        assert_eq!(
            check_recovered(dir.path(), low, high, &twin, &reference),
            low
        );
    }
    assert!(crashes > 20);
}

#[test]
fn test_open_without_marker_writes_nothing() {
    let dir = TempDir::new().unwrap();
    build(dir.path(), 2, 4);
    let before = dump_at(dir.path(), true);

    WRITES_BEFORE_CRASH.set(Some(1));
    let db = Brc20ProgDatabase::new(dir.path()).unwrap();
    WRITES_BEFORE_CRASH.set(None);
    assert!(!db.has_marker());
    assert_eq!(db.get_latest_block_height().unwrap(), 4);
    assert_eq!(db.dump(true), before);
}

#[test]
fn test_interrupted_first_commit_refuses_to_open() {
    let dir = TempDir::new().unwrap();
    let engine = open_engine(dir.path());
    execute_block(&engine, 0);
    assert!(crash_after(3, || engine.commit_to_db()));
    drop(engine);

    let error = Brc20ProgDatabase::new(dir.path())
        .err()
        .unwrap()
        .to_string();
    assert!(error.contains("first commit"), "{}", error);
}

#[test]
fn test_interrupted_commit_deeper_than_history_refuses_to_open() {
    let dir = TempDir::new().unwrap();
    let engine = open_engine(dir.path());
    execute_block(&engine, 0);
    engine.commit_to_db().unwrap();
    for n in 1..=MAX_REORG_HISTORY_SIZE + 1 {
        execute_block(&engine, n);
    }
    assert!(crash_after(3, || engine.commit_to_db()));
    drop(engine);

    let error = Brc20ProgDatabase::new(dir.path())
        .err()
        .unwrap()
        .to_string();
    assert!(
        error.contains("Rolling back to block 0 failed"),
        "{}",
        error
    );
}

async fn balances(engine: &BRC20ProgEngine, holders: &[(Bytes, Address)]) -> Vec<U256> {
    let mut balances = Vec::new();
    for (ticker, holder) in holders {
        let tx = load_brc20_balance_tx(ticker.clone(), *holder);
        let result = engine.read_contract(&tx, None, None).await.unwrap();
        balances.push(decode_brc20_balance_result(result.output.as_ref()));
    }
    balances
}

fn mint_block(engine: &BRC20ProgEngine, n: u64, holders: &[(Bytes, Address)]) {
    for (idx, (ticker, holder)) in holders.iter().enumerate() {
        let tx = load_brc20_mint_tx(ticker.clone(), *holder, U256::from(n));
        let id = format!("crash_test_{}i{}", n, idx);
        engine
            .add_tx_to_block(
                TIMESTAMP,
                &tx,
                idx as u64,
                n,
                B256::ZERO,
                id,
                1000,
                B256::ZERO,
            )
            .unwrap();
    }
    engine
        .finalise_block(TIMESTAMP, n, B256::ZERO, holders.len() as u64)
        .unwrap();
}

/// Runs against `PROG_SIGNET_DB`, a copy of a signet engine directory, which it changes: it
/// opens the copy as is, commits five blocks, then interrupts the commit of a sixth.
#[tokio::test]
#[ignore = "needs PROG_SIGNET_DB, a disposable copy of a signet engine directory"]
async fn test_signet_copy_recovers_interrupted_commit() {
    let path = std::env::var("PROG_SIGNET_DB").unwrap();
    let path = Path::new(&path);

    WRITES_BEFORE_CRASH.set(Some(1));
    let engine = BRC20ProgEngine::new(Brc20ProgDatabase::new(path).unwrap());
    WRITES_BEFORE_CRASH.set(None);
    let start = engine.get_latest_block_height().unwrap();

    let mut holders: Vec<(Bytes, Address)> = Vec::new();
    for n in (start.saturating_sub(20000)..=start).rev() {
        let Some(block) = engine.get_block_by_number(n, true).unwrap() else {
            continue;
        };
        for tx in block.transactions.right().unwrap_or_default() {
            if let Ok(call) = crate::brc20_controller::mintCall::abi_decode(&tx.input.bytes) {
                if !holders.contains(&(call._0.clone(), call._1)) {
                    holders.push((call._0, call._1));
                }
            }
        }
        if holders.len() >= 5 {
            break;
        }
    }
    eprintln!("start height {}, holders {:?}", start, holders);
    assert!(!holders.is_empty());

    for n in start + 1..=start + 5 {
        mint_block(&engine, n, &holders);
        WRITES_BEFORE_CRASH.set(Some(usize::MAX));
        let started = std::time::Instant::now();
        engine.commit_to_db().unwrap();
        let elapsed = started.elapsed();
        let writes = usize::MAX - WRITES_BEFORE_CRASH.get().unwrap();
        WRITES_BEFORE_CRASH.set(None);
        eprintln!("block {}: commit {:?}, {} batch writes", n, elapsed, writes);
    }

    let height = start + 5;
    let hashes: Vec<_> = (height - 20..=height)
        .map(|n| engine.get_block_by_number(n, false).unwrap().unwrap().hash)
        .collect();
    let before = balances(&engine, &holders).await;

    mint_block(&engine, height + 1, &holders);
    let interrupted_outputs = block_outputs(&engine, height + 1);
    assert!(crash_after(12, || engine.commit_to_db()));
    drop(engine);

    let started = std::time::Instant::now();
    let engine = BRC20ProgEngine::new(Brc20ProgDatabase::new(path).unwrap());
    eprintln!("open with rollback {:?}", started.elapsed());
    assert_eq!(engine.get_latest_block_height().unwrap(), height);
    let recovered: Vec<_> = (height - 20..=height)
        .map(|n| engine.get_block_by_number(n, false).unwrap().unwrap().hash)
        .collect();
    assert_eq!(recovered, hashes);
    assert_eq!(balances(&engine, &holders).await, before);
    eprintln!("recovered at {}, balances {:?}", height, before);

    mint_block(&engine, height + 1, &holders);
    assert_eq!(block_outputs(&engine, height + 1), interrupted_outputs);
    engine.commit_to_db().unwrap();
}
