//! A data dir written by brc20-prog v0.16.6 in the per-database layout migrates into the engine
//! store when a server opens it, with every key and value unchanged, and serves the consensus
//! fixture exactly like a fresh run.
//!
//! tests/fixtures/v0_16_6_mainnet_db is the mainnet consensus fixture workload run and committed
//! by brc20-prog v0.16.6, with the RocksDB LOG and LOCK files left out.

use std::fs;
use std::path::{Path, PathBuf};

use rocksdb::{IteratorMode, Options, DB};
use tempfile::TempDir;
use test_utils::{render_consensus_fixture_from, Era, FixtureSpec, CHAIN_ID_MAINNET};

fn fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/v0_16_6_mainnet_db")
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
}

type Pairs = Vec<(Vec<u8>, Vec<u8>)>;

fn collect<I>(iter: I) -> Pairs
where
    I: Iterator<Item = Result<(Box<[u8]>, Box<[u8]>), rocksdb::Error>>,
{
    iter.map(|item| {
        let (key, value) = item.unwrap();
        (key.to_vec(), value.to_vec())
    })
    .collect()
}

#[tokio::test]
async fn v0_16_6_data_dir_migrates_and_serves_the_consensus_fixture() {
    let fixture = fixture_path();
    let old_names: Vec<String> = fs::read_dir(&fixture)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    assert_eq!(old_names.len(), 29);

    let data_dir = TempDir::new().unwrap();
    copy_dir(&fixture, data_dir.path());

    let (fresh_report, migrated_report) = render_consensus_fixture_from(
        FixtureSpec {
            network: "bitcoin",
            chain_id: CHAIN_ID_MAINNET,
            eras: vec![Era {
                name: "mainnet cancun",
                start_height: None,
                transact_file: Some("consensus_fixture/fixture_txs_mainnet"),
            }],
        },
        data_dir.path(),
    )
    .await;
    assert_eq!(migrated_report, fresh_report);

    // The same report the mainnet consensus fixture pins.
    let snapshot = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/snapshots/consensus_fixture_mainnet__consensus_fixture_mainnet.snap"),
    )
    .unwrap();
    let pinned = snapshot.splitn(3, "---\n").nth(2).unwrap();
    assert_eq!(migrated_report.trim_end(), pinned.trim_end());

    // Only the engine store and its open lock are left, and each column family equals its
    // v0.16.6 database.
    let mut remaining: Vec<String> = fs::read_dir(data_dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    remaining.sort();
    assert_eq!(
        remaining,
        vec!["engine".to_string(), "engine.lock".to_string()]
    );

    let engine_path = data_dir.path().join("engine");
    let column_families = DB::list_cf(&Options::default(), &engine_path).unwrap();
    let engine =
        DB::open_cf_for_read_only(&Options::default(), &engine_path, &column_families, false)
            .unwrap();
    for name in &old_names {
        let source =
            DB::open_for_read_only(&Options::default(), fixture.join(name), false).unwrap();
        let expected = collect(source.full_iterator(IteratorMode::Start));
        let migrated =
            collect(engine.full_iterator_cf(engine.cf_handle(name).unwrap(), IteratorMode::Start));
        assert_eq!(migrated, expected, "column family {}", name);
    }
}
