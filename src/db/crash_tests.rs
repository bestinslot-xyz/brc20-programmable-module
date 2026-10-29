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

/// Copies the closed store at `from` into the empty directory `to`
fn copy_store(from: &Path, to: &Path) {
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            std::fs::create_dir(&target).unwrap();
            copy_store(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// Commits blocks `0..=low`, then crashes the commit of `low + 1..=high` after every write
fn commit_kill_case(low: u64, high: u64) {
    let base = TempDir::new().unwrap();
    build(base.path(), low, low);
    let twin = dump_at(base.path(), false);
    let reference = reference_run(low, high);

    let mut heights = Vec::new();
    for writes in 1.. {
        let dir = TempDir::new().unwrap();
        copy_store(base.path(), dir.path());
        let engine = open_engine(dir.path());
        for n in low + 1..=high {
            execute_block(&engine, n);
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
fn test_commit_crash_after_every_write_rolls_back() {
    // Holders first minted more than MAX_REORG_HISTORY_SIZE blocks back keep their history as
    // old ones
    commit_kill_case(12, 14);
}

#[test]
fn test_commit_of_ten_blocks_crash_after_every_write_rolls_back() {
    // The deepest commit the reorg history can undo
    commit_kill_case(12, 12 + MAX_REORG_HISTORY_SIZE);
}

#[test]
fn test_reorg_crash_after_every_write_rolls_back() {
    // The reorg undoes blocks 13..=15, whose new keys have a history old enough to delete
    let (low, high) = (12, 15);
    let twin_dir = TempDir::new().unwrap();
    build(twin_dir.path(), low, low);
    let twin = dump_at(twin_dir.path(), false);
    let reference = reference_run(low, high);
    let base = TempDir::new().unwrap();
    build(base.path(), low + 1, high);

    let mut crashes = 0;
    for writes in 1.. {
        let dir = TempDir::new().unwrap();
        copy_store(base.path(), dir.path());
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
fn test_reorg_above_the_committed_height_crash_rolls_back_to_it() {
    // Blocks 0..=14 are committed and 15..=17 only executed, then the reorg keeps 15 and 16
    let (committed, target, high) = (14, 16, 17);
    let base = TempDir::new().unwrap();
    build(base.path(), committed, committed);
    let twin = dump_at(base.path(), false);
    let reference = reference_run(committed, high);
    let target_dir = TempDir::new().unwrap();
    build(target_dir.path(), target, target);
    let at_target = dump_at(target_dir.path(), false);

    let mut heights = Vec::new();
    for writes in 1.. {
        let dir = TempDir::new().unwrap();
        copy_store(base.path(), dir.path());
        let engine = open_engine(dir.path());
        for n in committed + 1..=high {
            execute_block(&engine, n);
        }
        if !crash_after(writes, || engine.reorg(target)) {
            break;
        }
        drop(engine);
        let db = Brc20ProgDatabase::new(dir.path()).unwrap();
        let height = db.get_latest_block_height().unwrap();
        heights.push(height);
        if height == target {
            // The finished reorg also wrote the traces of block 17, which reorgs keep
            assert_same(&db.dump(false), &at_target);
        } else {
            drop(db);
            check_recovered(dir.path(), committed, high, &twin, &reference);
        }
    }
    assert_eq!(heights.pop().unwrap(), target);
    assert!(heights.len() > 20);
    assert!(heights.iter().all(|&h| h == committed));
}

#[test]
fn test_crash_during_rollback_recovers_on_next_open() {
    let (low, high) = (12, 14);
    let twin_dir = TempDir::new().unwrap();
    build(twin_dir.path(), low, low);
    let twin = dump_at(twin_dir.path(), false);
    let reference = reference_run(low, high);
    let crashed = TempDir::new().unwrap();
    copy_store(twin_dir.path(), crashed.path());
    let engine = open_engine(crashed.path());
    for n in low + 1..=high {
        execute_block(&engine, n);
    }
    assert!(crash_after(15, || engine.commit_to_db()));
    drop(engine);

    let mut crashes = 0;
    for writes in 1.. {
        let dir = TempDir::new().unwrap();
        copy_store(crashed.path(), dir.path());
        if !crash_after(writes, || Brc20ProgDatabase::new(dir.path()).map(|_| ())) {
            break;
        }
        crashes += 1;
        assert_eq!(
            check_recovered(dir.path(), low, high, &twin, &reference),
            low
        );
    }
    assert!(crashes > 20);
}

#[test]
fn test_reorg_below_a_replaced_chain_errors_and_keeps_the_store() {
    let dir = TempDir::new().unwrap();
    build(dir.path(), 30, 30);
    let engine = open_engine(dir.path());
    engine.reorg(20).unwrap();
    execute_block(&engine, 21);
    engine.commit_to_db().unwrap();

    // The histories were pruned against block 30, so they cannot reach block 11
    let error = engine.reorg(11).err().unwrap().to_string();
    assert!(error.contains("max recorded block height"), "{}", error);
    drop(engine);

    let db = Brc20ProgDatabase::new(dir.path()).unwrap();
    assert!(!db.has_marker());
    assert_eq!(db.get_latest_block_height().unwrap(), 21);
}

#[test]
fn test_history_too_short_for_the_rollback_wipes_the_store() {
    let dir = TempDir::new().unwrap();
    build(dir.path(), 30, 30);
    let engine = open_engine(dir.path());
    engine.reorg(20).unwrap();
    execute_block(&engine, 21);
    engine.commit_to_db().unwrap();
    drop(engine);

    // An older version lowered the highest finalised block on replay, which let a reorg past
    // the pruned history through the depth check
    let mut db = Brc20ProgDatabase::new(dir.path()).unwrap();
    let global = db.db_global_values.as_mut().unwrap();
    global
        .set(MAX_BLOCK_NUMBER_KEY.to_string(), "21".to_string())
        .unwrap();
    let error = db.reorg(11).err().unwrap();
    assert!(error.is::<HistoryTooShort>(), "{}", error);
    assert!(db.has_marker());
    drop(db);

    assert_wiped_and_working(dir.path());
}

#[test]
fn test_reorg_after_losing_uncommitted_blocks_stays_allowed() {
    // Blocks 13..=21 are finalised but never committed, then the process dies
    let dir = TempDir::new().unwrap();
    build(dir.path(), 12, 12);
    let engine = open_engine(dir.path());
    for n in 13..=21 {
        execute_block(&engine, n);
    }
    drop(engine);

    let engine = open_engine(dir.path());
    assert_eq!(engine.get_latest_block_height().unwrap(), 12);
    execute_block(&engine, 13);
    engine.commit_to_db().unwrap();
    engine.reorg(7).unwrap();
    assert_eq!(engine.get_latest_block_height().unwrap(), 7);
}

#[test]
fn test_failed_commit_blocks_later_writes_until_reopen() {
    let dir = TempDir::new().unwrap();
    build(dir.path(), 12, 12);
    let engine = open_engine(dir.path());
    for n in 13..=14 {
        execute_block(&engine, n);
    }
    assert!(crash_after(10, || engine.commit_to_db()));

    // Writes and reads alike, including a reorg to the current height and a block
    let results = [
        engine.commit_to_db().err(),
        engine.reorg(12).err(),
        engine.reorg(14).err(),
        engine.finalise_block(TIMESTAMP, 15, B256::ZERO, 0).err(),
        engine.get_latest_block_height().err(),
        engine.get_block_by_number(12, false).err(),
        engine.get_transaction_count(holder(1), 12).err(),
        engine.get_storage_at(holder(1), U256::ZERO).err(),
    ];
    for error in results {
        let error = error.unwrap().to_string();
        assert!(error.contains("reopen the database"), "{}", error);
    }
    drop(engine);

    let db = Brc20ProgDatabase::new(dir.path()).unwrap();
    assert!(!db.has_marker());
    assert_eq!(db.get_latest_block_height().unwrap(), 12);
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

/// Leaves `path` with an interrupted first commit, which no rollback can undo
fn interrupt_first_commit(path: &Path) {
    let engine = open_engine(path);
    execute_block(&engine, 0);
    assert!(crash_after(3, || engine.commit_to_db()));
}

/// Leaves `path` with an interrupted commit of 11 blocks, deeper than the reorg history
fn interrupt_deep_commit(path: &Path) {
    let engine = open_engine(path);
    execute_block(&engine, 0);
    engine.commit_to_db().unwrap();
    for n in 1..=MAX_REORG_HISTORY_SIZE + 1 {
        execute_block(&engine, n);
    }
    assert!(crash_after(3, || engine.commit_to_db()));
}

/// Opens `path`, checks it is an empty store, then checks blocks commit into it
fn assert_wiped_and_working(path: &Path) {
    let db = Brc20ProgDatabase::new(path).unwrap();
    assert!(!db.has_marker());
    assert_eq!(db.get_latest_block_height().unwrap(), 0);
    assert!(db.get_block(0).unwrap().is_none());
    assert!(db.dump(true).iter().all(|(_, entries)| entries.is_empty()));
    assert_eq!(
        max_block_number(db.db_global_values.as_ref().unwrap()).unwrap(),
        0
    );
    drop(db);

    build(path, 2, 2);
    assert_eq!(
        Brc20ProgDatabase::new(path)
            .unwrap()
            .get_latest_block_height()
            .unwrap(),
        2
    );
}

#[test]
fn test_interrupted_first_commit_wipes_the_store() {
    let dir = TempDir::new().unwrap();
    interrupt_first_commit(dir.path());
    assert_wiped_and_working(dir.path());
}

#[test]
fn test_interrupted_commit_deeper_than_history_wipes_the_store() {
    let dir = TempDir::new().unwrap();
    interrupt_deep_commit(dir.path());
    assert_wiped_and_working(dir.path());
}

#[test]
fn test_interrupted_wipe_completes_on_next_open() {
    let dir = TempDir::new().unwrap();
    interrupt_deep_commit(dir.path());

    // The wipe stops after setting its marker. Then one database loses its CURRENT file and
    // another goes entirely, as a crash partway through the deletions leaves them.
    assert!(crash_after(1, || Brc20ProgDatabase::new(dir.path()).map(|_| ())));
    std::fs::remove_file(dir.path().join("account").join("CURRENT")).unwrap();
    std::fs::remove_dir_all(dir.path().join("code")).unwrap();

    assert_wiped_and_working(dir.path());
}

#[test]
fn test_wipe_keeps_files_it_does_not_own() {
    let dir = TempDir::new().unwrap();
    interrupt_first_commit(dir.path());
    let mut config = ConfigDatabase::new(dir.path(), "config").unwrap();
    config.set("key".to_string(), "value".to_string()).unwrap();
    drop(config);
    std::fs::write(dir.path().join("notes.txt"), "keep").unwrap();

    assert_wiped_and_working(dir.path());
    let config = ConfigDatabase::new(dir.path(), "config").unwrap();
    assert_eq!(
        config.get("key".to_string()).unwrap().as_deref(),
        Some("value")
    );
    assert_eq!(
        std::fs::read_to_string(dir.path().join("notes.txt")).unwrap(),
        "keep"
    );
}

/// Moves the `account` database of `dir` into `elsewhere` and links it back
#[cfg(unix)]
fn link_account_database(dir: &Path, elsewhere: &Path) -> std::path::PathBuf {
    let linked = elsewhere.join("account");
    std::fs::rename(dir.join("account"), &linked).unwrap();
    std::os::unix::fs::symlink(&linked, dir.join("account")).unwrap();
    std::fs::write(elsewhere.join("notes.txt"), "keep").unwrap();
    linked
}

#[cfg(unix)]
#[test]
fn test_wipe_empties_a_symlinked_database_and_keeps_the_link() {
    let dir = TempDir::new().unwrap();
    let elsewhere = TempDir::new().unwrap();
    interrupt_first_commit(dir.path());
    let linked = link_account_database(dir.path(), elsewhere.path());

    assert_wiped_and_working(dir.path());
    assert!(is_symlink(&dir.path().join("account")));
    assert!(linked.join("CURRENT").exists());
    assert!(elsewhere.path().join("notes.txt").exists());
}

#[cfg(unix)]
#[test]
fn test_interrupted_emptying_of_a_symlinked_database_completes() {
    let dir = TempDir::new().unwrap();
    let elsewhere = TempDir::new().unwrap();
    interrupt_first_commit(dir.path());
    let linked = link_account_database(dir.path(), elsewhere.path());

    // The wipe stops after setting its marker, then the target loses a file, so it no
    // longer looks like a database
    assert!(crash_after(1, || Brc20ProgDatabase::new(dir.path()).map(|_| ())));
    std::fs::remove_file(linked.join("IDENTITY")).unwrap();

    assert_wiped_and_working(dir.path());
    assert!(is_symlink(&dir.path().join("account")));
}

#[cfg(unix)]
#[test]
fn test_wipe_stops_at_a_link_to_something_else() {
    let dir = TempDir::new().unwrap();
    let elsewhere = TempDir::new().unwrap();
    interrupt_first_commit(dir.path());
    std::fs::remove_dir_all(dir.path().join("account")).unwrap();
    std::fs::write(elsewhere.path().join("notes.txt"), "keep").unwrap();
    std::os::unix::fs::symlink(elsewhere.path(), dir.path().join("account")).unwrap();

    let error = Brc20ProgDatabase::new(dir.path())
        .err()
        .unwrap()
        .to_string();
    assert!(error.contains("not a database"), "{}", error);
    assert!(elsewhere.path().join("notes.txt").exists());
    assert!(dir.path().join("code").join("CURRENT").exists());
}

#[test]
fn test_wipe_list_names_every_database_directory() {
    let dir = TempDir::new().unwrap();
    drop(Brc20ProgDatabase::new(dir.path()).unwrap());
    let mut created: Vec<String> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter(|name| name != "global")
        .collect();
    created.sort();
    let mut listed: Vec<String> = DATABASE_DIRS.iter().map(|name| name.to_string()).collect();
    listed.sort();
    assert_eq!(created, listed);
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
