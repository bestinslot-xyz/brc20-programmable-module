//! Brc20Prog::open_with_bitcoin_rpc routes the Bitcoin precompiles through a caller-supplied
//! BitcoinRpc instead of a Bitcoin Core node.
//!
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard};

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
use brc20_prog::types::{EthCall, RawBytes};
use brc20_prog::{BitcoinRpc, BitcoinRpcError, Brc20Prog, Brc20ProgApiServer, Brc20ProgConfig};
use tempfile::TempDir;
use test_utils::get_free_port;

sol! {
    function getTxDetails(bytes32 txid) returns (uint256 block_height, bytes32[] vin_txids, uint256[] vin_vouts , bytes[] vin_scriptPubKeys, uint256[] vin_values, bytes[] vout_scriptPubKeys, uint256[] vout_values);
}

const TX_DETAILS_PRECOMPILE: [u8; 20] = {
    let mut address = [0u8; 20];
    address[19] = 0xfd;
    address
};

#[derive(Default)]
struct FakeBitcoinRpc {
    transactions: Mutex<HashMap<Txid, (Transaction, Option<BlockHash>)>>,
    heights: Mutex<HashMap<BlockHash, usize>>,
    /// Transient failures still to return per txid before answering
    failures_left: Mutex<HashMap<Txid, u32>>,
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

    fn take_failure(&self, txid: &Txid) -> Result<(), BitcoinRpcError> {
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

async fn get_tx_details(prog: &Brc20Prog, txid: &Txid) -> Result<String, String> {
    let data = getTxDetailsCall::new((to_b256(txid),)).abi_encode();
    prog.eth_call(
        EthCall::new(
            Some([1u8; 20].into()),
            Some(TX_DETAILS_PRECOMPILE.into()),
            RawBytes::new(format!("0x{}", hex::encode(data))),
        ),
        Some("latest".to_string()),
    )
    .await
    .map_err(|error| error.to_string())
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
async fn test_transient_errors_are_retried() {
    let fake = Arc::new(FakeBitcoinRpc::default());
    let embedded = open_with_fake("signet", fake.clone());

    let prev = transaction(vec![OutPoint::null()], vec![(7_000, &[0x51, 0x05])]);
    let prev_txid = fake.add(prev, Some((block_hash(3), 2)));
    let tx = transaction(
        vec![OutPoint::new(prev_txid, 0)],
        vec![(6_000, &[0x51, 0x06])],
    );
    let txid = fake.add(tx, Some((block_hash(4), 2)));
    fake.failures_left.lock().unwrap().insert(txid, 2);

    embedded.prog.brc20_mine(10, 0).await.unwrap();

    let output = get_tx_details(&embedded.prog, &txid).await.unwrap();
    let details = getTxDetailsCall::abi_decode_returns(&hex::decode(output).unwrap()).unwrap();

    assert_eq!(details.block_height, U256::from(2));
    assert_eq!(fake.failures_left.lock().unwrap()[&txid], 0);
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
