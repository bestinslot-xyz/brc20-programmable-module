//! Brc20Prog::open_with_bitcoin_rpc routes the Bitcoin precompiles through a caller-supplied
//! BitcoinRpc instead of a Bitcoin Core node.
//!
//! A BitcoinRpc error fails a public call after one immediate retry, and panics during block
//! execution after five retries one second apart.
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use alloy::primitives::{hex, Bytes, B256, U256};
use alloy::sol;
use alloy::sol_types::SolCall;
use bitcoin::absolute::LockTime;
use bitcoin::hashes::Hash;
use bitcoin::transaction::Version;
use bitcoin::{
    Amount, BlockHash, Network, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid,
    Witness,
};
use brc20_prog::types::{EthCall, RawBytes, TxReceiptED};
use brc20_prog::{BitcoinRpc, BitcoinRpcError, Brc20Prog, Brc20ProgApiServer, Brc20ProgConfig};
use tempfile::TempDir;
use test_utils::get_free_port;

sol! {
    function getTxDetails(bytes32 txid) returns (uint256 block_height, bytes32[] vin_txids, uint256[] vin_vouts , bytes[] vin_scriptPubKeys, uint256[] vin_values, bytes[] vout_scriptPubKeys, uint256[] vout_values);
    function getLastSatLocation(bytes32 txid, uint256 vout, uint256 sat) returns (bytes32 last_txid, uint256 last_vout, uint256 last_sat, bytes old_pkscript, bytes new_pkscript);
}

const TX_DETAILS_PRECOMPILE: [u8; 20] = precompile_address(0xfd);
const LAST_SAT_LOCATION_PRECOMPILE: [u8; 20] = precompile_address(0xfc);

const fn precompile_address(last_byte: u8) -> [u8; 20] {
    let mut address = [0u8; 20];
    address[19] = last_byte;
    address
}

/// Error a public call returns when a Bitcoin lookup fails
const UNAVAILABLE: &str = "Bitcoin data unavailable";

/// A public call that hits a failing lookup answers well inside this
const PUBLIC_CALL_DEADLINE: Duration = Duration::from_millis(500);

/// Lookups that fail until the test ends
const ALWAYS: u32 = u32::MAX;

#[derive(Default)]
struct FakeBitcoinRpc {
    transactions: Mutex<HashMap<Txid, (Transaction, Option<BlockHash>)>>,
    heights: Mutex<HashMap<BlockHash, usize>>,
    /// Transient failures still to return per txid before answering
    failures_left: Mutex<HashMap<Txid, u32>>,
    /// Lookups per txid that answer before the failures in `failures_left` start
    answers_before_failing: Mutex<HashMap<Txid, u32>>,
}

impl FakeBitcoinRpc {
    fn add(&self, tx: Transaction, block: Option<(BlockHash, usize)>) -> Txid {
        let txid = tx.compute_txid();
        if let Some((hash, height)) = block {
            self.heights.lock().unwrap().insert(hash, height);
        }
        self.transactions
            .lock()
            .unwrap()
            .insert(txid, (tx, block.map(|(hash, _)| hash)));
        txid
    }

    /// Adds `tx` as confirmed in a block whose height lookup fails
    fn add_in_unknown_block(&self, tx: Transaction, block: BlockHash) -> Txid {
        let txid = tx.compute_txid();
        self.transactions
            .lock()
            .unwrap()
            .insert(txid, (tx, Some(block)));
        txid
    }

    fn fail(&self, txid: Txid, times: u32) {
        self.failures_left.lock().unwrap().insert(txid, times);
    }

    fn failures_left(&self, txid: &Txid) -> u32 {
        self.failures_left.lock().unwrap()[txid]
    }

    /// Answers the next `answers` lookups of `txid`, then fails the next `failures`
    fn fail_later(&self, txid: Txid, answers: u32, failures: u32) {
        self.answers_before_failing
            .lock()
            .unwrap()
            .insert(txid, answers);
        self.fail(txid, failures);
    }

    fn take_failure(&self, txid: &Txid) -> Result<(), BitcoinRpcError> {
        if let Some(answers) = self.answers_before_failing.lock().unwrap().get_mut(txid) {
            if *answers > 0 {
                *answers -= 1;
                return Ok(());
            }
        }
        let mut failures = self.failures_left.lock().unwrap();
        match failures.get_mut(txid) {
            Some(left) if *left > 0 => {
                *left -= 1;
                Err("connection reset".into())
            }
            _ => Ok(()),
        }
    }
}

impl BitcoinRpc for FakeBitcoinRpc {
    fn get_transaction(&self, txid: &Txid) -> Result<Option<Transaction>, BitcoinRpcError> {
        self.take_failure(txid)?;
        Ok(self
            .transactions
            .lock()
            .unwrap()
            .get(txid)
            .map(|(tx, _)| tx.clone()))
    }

    fn get_transaction_with_block_hash(
        &self,
        txid: &Txid,
    ) -> Result<Option<(Transaction, Option<BlockHash>)>, BitcoinRpcError> {
        self.take_failure(txid)?;
        Ok(self.transactions.lock().unwrap().get(txid).cloned())
    }

    fn get_block_height(&self, block_hash: &BlockHash) -> Result<usize, BitcoinRpcError> {
        self.heights
            .lock()
            .unwrap()
            .get(block_hash)
            .copied()
            .ok_or_else(|| "unknown block".into())
    }

    fn network(&self) -> Result<Network, BitcoinRpcError> {
        Ok(Network::Signet)
    }
}

struct Embedded {
    prog: Brc20Prog,
    _db_path: TempDir,
    _exclusive: MutexGuard<'static, ()>,
}

fn open_with_fake(network: &str, fake: Arc<FakeBitcoinRpc>) -> Embedded {
    // The config and the Bitcoin source are process-wide, so tests take turns
    static EXCLUSIVE: Mutex<()> = Mutex::new(());
    let exclusive = EXCLUSIVE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let db_path = TempDir::new().unwrap();
    let prog = Brc20Prog::open_with_bitcoin_rpc(
        Brc20ProgConfig {
            db_path: db_path.path().to_str().unwrap().to_string(),
            brc20_prog_rpc_server_url: format!("127.0.0.1:{}", get_free_port()),
            // No Bitcoin Core URL: the injected source replaces it
            bitcoin_rpc_url: String::new(),
            bitcoin_rpc_network: network.to_string(),
            fail_on_bitcoin_rpc_error: true,
            evm_record_traces: true,
            ..Default::default()
        },
        fake,
    )
    .expect("Failed to open brc20-prog");
    Embedded {
        prog,
        _db_path: db_path,
        _exclusive: exclusive,
    }
}

fn transaction(inputs: Vec<OutPoint>, outputs: Vec<(u64, &[u8])>) -> Transaction {
    Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: inputs
            .into_iter()
            .map(|previous_output| TxIn {
                previous_output,
                script_sig: ScriptBuf::new(),
                sequence: Sequence::MAX,
                witness: Witness::new(),
            })
            .collect(),
        output: outputs
            .into_iter()
            .map(|(value, script)| TxOut {
                value: Amount::from_sat(value),
                script_pubkey: ScriptBuf::from_bytes(script.to_vec()),
            })
            .collect(),
    }
}

fn block_hash(seed: u8) -> BlockHash {
    BlockHash::from_byte_array([seed; 32])
}

fn to_b256(txid: &Txid) -> B256 {
    B256::from_str(&txid.to_string()).unwrap()
}

fn eth_call_to(precompile: [u8; 20], data: &[u8]) -> EthCall {
    EthCall::new(
        Some([1u8; 20].into()),
        Some(precompile.into()),
        RawBytes::new(format!("0x{}", hex::encode(data))),
    )
}

fn tx_details_data(txid: &Txid) -> Vec<u8> {
    getTxDetailsCall::new((to_b256(txid),)).abi_encode()
}

fn last_sat_location_data(txid: &Txid, vout: u64, sat: u64) -> Vec<u8> {
    getLastSatLocationCall::new((to_b256(txid), U256::from(vout), U256::from(sat))).abi_encode()
}

async fn eth_call(prog: &Brc20Prog, precompile: [u8; 20], data: &[u8]) -> Result<String, String> {
    prog.eth_call(eth_call_to(precompile, data), Some("latest".to_string()))
        .await
        .map_err(|error| error.to_string())
}

async fn get_tx_details(prog: &Brc20Prog, txid: &Txid) -> Result<String, String> {
    eth_call(prog, TX_DETAILS_PRECOMPILE, &tx_details_data(txid)).await
}

/// Runs `data` against `precompile` as a transaction in the next block
async fn call_in_block(prog: &Brc20Prog, precompile: [u8; 20], data: &[u8]) -> TxReceiptED {
    prog.brc20_call(
        "aaaaabbbbccccddddeeeeffff00001111222233333".to_string(),
        Some(precompile.into()),
        None,
        Some(RawBytes::new(format!("0x{}", hex::encode(data)))),
        None,
        12345,
        [0u8; 32].into(),
        0,
        "inscription".to_string(),
        1_000,
        [0u8; 32].into(),
    )
    .await
    .unwrap()
    .unwrap()
}

/// Asserts that a public call failed on the Bitcoin lookup, quickly
fn assert_unavailable(result: Result<String, String>, started: Instant) {
    let error = result.unwrap_err();
    assert!(error.contains(UNAVAILABLE), "{error}");
    assert!(
        started.elapsed() < PUBLIC_CALL_DEADLINE,
        "took {:?}",
        started.elapsed()
    );
}

/// A confirmed transaction spending one confirmed output, both known to `fake`.
/// Returns (prev_txid, txid).
fn spend(fake: &FakeBitcoinRpc, seed: u8) -> (Txid, Txid) {
    let prev = transaction(vec![OutPoint::null()], vec![(5_000, &[0x51, seed])]);
    let prev_txid = fake.add(prev, Some((block_hash(seed), 1)));
    let tx = transaction(
        vec![OutPoint::new(prev_txid, 0)],
        vec![(3_000, &[0x52, seed]), (1_500, &[0x53, seed])],
    );
    let txid = fake.add(tx, Some((block_hash(seed + 1), 2)));
    (prev_txid, txid)
}

#[tokio::test]
async fn test_tx_details_come_from_injected_source() {
    let fake = Arc::new(FakeBitcoinRpc::default());
    let embedded = open_with_fake("signet", fake.clone());

    let prev = transaction(vec![OutPoint::null()], vec![(5_000, &[0x51, 0x01])]);
    let prev_txid = fake.add(prev, Some((block_hash(1), 3)));
    let tx = transaction(
        vec![OutPoint::new(prev_txid, 0)],
        vec![(3_000, &[0x51, 0x02]), (1_500, &[0x51, 0x03])],
    );
    let txid = fake.add(tx, Some((block_hash(2), 4)));

    embedded.prog.brc20_mine(10, 0).await.unwrap();

    let output = get_tx_details(&embedded.prog, &txid).await.unwrap();
    let details = getTxDetailsCall::abi_decode_returns(&hex::decode(output).unwrap()).unwrap();

    assert_eq!(details.block_height, U256::from(4));
    assert_eq!(details.vin_txids, vec![to_b256(&prev_txid)]);
    assert_eq!(details.vin_vouts, vec![U256::ZERO]);
    assert_eq!(details.vin_scriptPubKeys, [Bytes::from([0x51, 0x01])]);
    assert_eq!(details.vin_values, vec![U256::from(5_000)]);
    assert_eq!(
        details.vout_scriptPubKeys,
        [Bytes::from([0x51, 0x02]), Bytes::from([0x51, 0x03])]
    );
    assert_eq!(
        details.vout_values,
        vec![U256::from(3_000), U256::from(1_500)]
    );
}

#[tokio::test]
async fn test_unknown_transaction_fails_the_call() {
    let embedded = open_with_fake("signet", Arc::new(FakeBitcoinRpc::default()));
    embedded.prog.brc20_mine(10, 0).await.unwrap();

    let unknown = transaction(vec![OutPoint::null()], vec![(1, &[0x51, 0x04])]);
    let result = get_tx_details(&embedded.prog, &unknown.compute_txid()).await;

    let error = result.unwrap_err();
    assert!(error.contains("PrecompileError"), "{error}");
}

#[tokio::test]
async fn test_transaction_above_current_height_fails_the_call() {
    let fake = Arc::new(FakeBitcoinRpc::default());
    let embedded = open_with_fake("signet", fake.clone());

    let prev = transaction(vec![OutPoint::null()], vec![(2_000, &[0x51, 0x07])]);
    let prev_txid = fake.add(prev, Some((block_hash(5), 1)));
    let tx = transaction(
        vec![OutPoint::new(prev_txid, 0)],
        vec![(1_000, &[0x51, 0x08])],
    );
    let txid = fake.add(tx, Some((block_hash(6), 50)));

    embedded.prog.brc20_mine(10, 0).await.unwrap();

    let error = get_tx_details(&embedded.prog, &txid).await.unwrap_err();
    assert!(error.contains("PrecompileError"), "{error}");

    // The same call succeeds once the chain reaches the transaction's height
    embedded.prog.brc20_mine(50, 0).await.unwrap();
    assert!(get_tx_details(&embedded.prog, &txid).await.is_ok());
}

#[tokio::test]
async fn test_last_sat_location_comes_from_injected_source() {
    let fake = Arc::new(FakeBitcoinRpc::default());
    let embedded = open_with_fake("signet", fake.clone());
    let (prev_txid, txid) = spend(&fake, 0x20);
    embedded.prog.brc20_mine(10, 0).await.unwrap();

    let output = eth_call(
        &embedded.prog,
        LAST_SAT_LOCATION_PRECOMPILE,
        &last_sat_location_data(&txid, 1, 100),
    )
    .await
    .unwrap();
    let location =
        getLastSatLocationCall::abi_decode_returns(&hex::decode(output).unwrap()).unwrap();

    assert_eq!(location.last_txid, to_b256(&prev_txid));
    assert_eq!(location.last_vout, U256::ZERO);
    assert_eq!(location.last_sat, U256::from(3_100));
    assert_eq!(location.old_pkscript, Bytes::from([0x51, 0x20]));
    assert_eq!(location.new_pkscript, Bytes::from([0x53, 0x20]));
}

/// Every Bitcoin lookup of both Bitcoin precompiles, failing during an eth_call
#[tokio::test]
async fn test_failed_lookup_fails_a_public_call_at_once() {
    let fake = Arc::new(FakeBitcoinRpc::default());
    let embedded = open_with_fake("signet", fake.clone());
    embedded.prog.brc20_mine(10, 0).await.unwrap();

    // The transaction itself
    let (_, tx_lookup_fails) = spend(&fake, 0x30);
    fake.fail(tx_lookup_fails, ALWAYS);

    // The height of its block
    let (prev_txid, _) = spend(&fake, 0x40);
    let in_unknown_block = transaction(
        vec![OutPoint::new(prev_txid, 0)],
        vec![(1_000, &[0x51, 0x41])],
    );
    let height_lookup_fails = fake.add_in_unknown_block(in_unknown_block, block_hash(0x4f));

    // The transaction an input spends
    let (vin_lookup_fails_prev, vin_lookup_fails) = spend(&fake, 0x50);
    fake.fail(vin_lookup_fails_prev, ALWAYS);

    for txid in [tx_lookup_fails, height_lookup_fails, vin_lookup_fails] {
        let started = Instant::now();
        assert_unavailable(get_tx_details(&embedded.prog, &txid).await, started);

        let started = Instant::now();
        let result = eth_call(
            &embedded.prog,
            LAST_SAT_LOCATION_PRECOMPILE,
            &last_sat_location_data(&txid, 0, 0),
        )
        .await;
        assert_unavailable(result, started);
    }
}

#[tokio::test]
async fn test_failed_lookup_fails_estimate_gas_and_call_many() {
    let fake = Arc::new(FakeBitcoinRpc::default());
    let embedded = open_with_fake("signet", fake.clone());
    embedded.prog.brc20_mine(10, 0).await.unwrap();
    let (_, txid) = spend(&fake, 0x60);
    fake.fail(txid, ALWAYS);
    let call = || eth_call_to(TX_DETAILS_PRECOMPILE, &tx_details_data(&txid));
    let latest = || Some("latest".to_string());

    let started = Instant::now();
    let result = embedded.prog.eth_estimate_gas(call(), latest()).await;
    assert_unavailable(result.map_err(|error| error.to_string()), started);

    let started = Instant::now();
    let result = embedded
        .prog
        .eth_call_many(vec![call()], latest(), None)
        .await;
    assert_unavailable(
        result
            .map(|outputs| outputs.join(","))
            .map_err(|error| error.to_string()),
        started,
    );

    let started = Instant::now();
    let result = embedded
        .prog
        .eth_estimate_gas_many(vec![call()], latest(), None)
        .await;
    assert_unavailable(
        result
            .map(|outputs| outputs.join(","))
            .map_err(|error| error.to_string()),
        started,
    );
}

/// The binary search stops at a Bitcoin failure instead of reading it as too little gas
#[tokio::test]
async fn test_failed_lookup_mid_search_fails_estimate_gas() {
    let fake = Arc::new(FakeBitcoinRpc::default());
    let embedded = open_with_fake("signet", fake.clone());
    embedded.prog.brc20_mine(10, 0).await.unwrap();
    let latest = || Some("latest".to_string());

    // The first run answers, the first run of the search fails both attempts
    let (_, txid) = spend(&fake, 0x64);
    fake.fail_later(txid, 1, 2);
    let call = eth_call_to(TX_DETAILS_PRECOMPILE, &tx_details_data(&txid));
    let started = Instant::now();
    let result = embedded.prog.eth_estimate_gas(call, latest()).await;
    assert_unavailable(result.map_err(|error| error.to_string()), started);
    assert_eq!(fake.failures_left(&txid), 0);

    let (_, txid) = spend(&fake, 0x66);
    fake.fail_later(txid, 1, 2);
    let call = eth_call_to(TX_DETAILS_PRECOMPILE, &tx_details_data(&txid));
    let started = Instant::now();
    let result = embedded
        .prog
        .eth_estimate_gas_many(vec![call], latest(), None)
        .await;
    assert_unavailable(
        result
            .map(|outputs| outputs.join(","))
            .map_err(|error| error.to_string()),
        started,
    );
    assert_eq!(fake.failures_left(&txid), 0);
}

/// A public call retries a failed lookup once, with no pause
#[tokio::test]
async fn test_public_call_retries_once() {
    let fake = Arc::new(FakeBitcoinRpc::default());
    let embedded = open_with_fake("signet", fake.clone());
    embedded.prog.brc20_mine(10, 0).await.unwrap();
    let (_, txid) = spend(&fake, 0x70);

    fake.fail(txid, 1);
    let started = Instant::now();
    let output = get_tx_details(&embedded.prog, &txid).await.unwrap();
    assert!(started.elapsed() < PUBLIC_CALL_DEADLINE);
    let details = getTxDetailsCall::abi_decode_returns(&hex::decode(output).unwrap()).unwrap();
    assert_eq!(details.block_height, U256::from(2));
    assert_eq!(fake.failures_left(&txid), 0);

    fake.fail(txid, 2);
    let started = Instant::now();
    assert_unavailable(get_tx_details(&embedded.prog, &txid).await, started);
    assert_eq!(fake.failures_left(&txid), 0);
}

/// Block execution retries five times, so five failures followed by an answer still succeed
#[tokio::test]
async fn test_block_execution_retries_transient_errors() {
    let fake = Arc::new(FakeBitcoinRpc::default());
    let embedded = open_with_fake("signet", fake.clone());
    embedded.prog.brc20_mine(10, 0).await.unwrap();
    let (_, txid) = spend(&fake, 0x80);
    fake.fail(txid, 5);

    let receipt = call_in_block(
        &embedded.prog,
        TX_DETAILS_PRECOMPILE,
        &tx_details_data(&txid),
    )
    .await;

    assert_eq!(receipt.status, 1u8.into());
    assert_eq!(fake.failures_left(&txid), 0);
    let trace = embedded
        .prog
        .debug_trace_transaction(receipt.transaction_hash)
        .await
        .unwrap()
        .unwrap();
    let details = getTxDetailsCall::abi_decode_returns(&trace.output.bytes).unwrap();
    assert_eq!(details.block_height, U256::from(2));
}

/// Runs the tx details lookup as a block transaction on a fresh module, with `failures`
/// transient failures first. Returns the receipt's gas used and the trace as JSON.
async fn block_call_after_failures(failures: u32) -> (u64, String) {
    let fake = Arc::new(FakeBitcoinRpc::default());
    let embedded = open_with_fake("signet", fake.clone());
    embedded.prog.brc20_mine(10, 0).await.unwrap();
    let (_, txid) = spend(&fake, 0xa0);
    fake.fail(txid, failures);

    let receipt = call_in_block(
        &embedded.prog,
        TX_DETAILS_PRECOMPILE,
        &tx_details_data(&txid),
    )
    .await;
    assert_eq!(fake.failures_left(&txid), 0);
    let trace = embedded
        .prog
        .debug_trace_transaction(receipt.transaction_hash)
        .await
        .unwrap()
        .unwrap();
    (
        receipt.gas_used.uint.to::<u64>(),
        serde_json::to_string(&trace).unwrap(),
    )
}

/// A lookup that fails and then recovers leaves no mark on the block result
#[tokio::test]
async fn test_recovered_lookup_matches_clean_block_execution() {
    let clean = block_call_after_failures(0).await;
    let recovered = block_call_after_failures(1).await;

    assert_eq!(recovered.0, clean.0);
    assert_eq!(recovered.1, clean.1);
}

#[tokio::test]
#[should_panic(expected = "Bitcoin RPC unreachable")]
async fn test_failed_lookup_panics_during_block_execution() {
    let fake = Arc::new(FakeBitcoinRpc::default());
    let embedded = open_with_fake("signet", fake.clone());
    embedded.prog.brc20_mine(10, 0).await.unwrap();
    let (_, txid) = spend(&fake, 0x90);
    fake.fail(txid, ALWAYS);

    call_in_block(
        &embedded.prog,
        TX_DETAILS_PRECOMPILE,
        &tx_details_data(&txid),
    )
    .await;
}

#[tokio::test]
async fn test_initialise_checks_injected_network() {
    {
        let matching = open_with_fake("signet", Arc::new(FakeBitcoinRpc::default()));
        assert!(matching
            .prog
            .brc20_initialise([0u8; 32].into(), 0, 0)
            .await
            .is_ok());
    }

    let mismatched = open_with_fake("mainnet", Arc::new(FakeBitcoinRpc::default()));
    let error = mismatched
        .prog
        .brc20_initialise([0u8; 32].into(), 0, 0)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("network mismatch"), "{error}");
}
