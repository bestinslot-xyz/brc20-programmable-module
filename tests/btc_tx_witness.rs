//! BTC_TxWitness (0xf9) through the engine, with a fake Bitcoin source: confirmation and height
//! rules, gas, fork gating, and the same lookup failure behaviour as the other Bitcoin precompiles.
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
use brc20_prog::types::{EthCall, RawBytes, TxReceiptED};
use brc20_prog::{BitcoinRpc, BitcoinRpcError, Brc20Prog, Brc20ProgApiServer, Brc20ProgConfig};
use tempfile::TempDir;
use test_utils::get_free_port;

sol! {
    function getTxWitness(bytes32 txid, uint256 vin) returns (uint256 block_height, uint256 vin_count, bytes[] stack);
}

const WITNESS_PRECOMPILE: [u8; 20] = {
    let mut address = [0u8; 20];
    address[19] = 0xf9;
    address
};
const GAS_PER_RPC: u64 = 400_000;
const GAS_PER_TX_BYTE: u64 = 16;

struct FakeBitcoinRpc {
    network: Network,
    transactions: Mutex<HashMap<Txid, (Transaction, Option<BlockHash>)>>,
    heights: Mutex<HashMap<BlockHash, usize>>,
    failing: Mutex<bool>,
}

impl FakeBitcoinRpc {
    fn new(network: Network) -> Self {
        Self {
            network,
            transactions: Default::default(),
            heights: Default::default(),
            failing: Mutex::new(false),
        }
    }

    fn add(&self, tx: Transaction, block: Option<(u8, usize)>) -> Txid {
        let txid = tx.compute_txid();
        let hash = block.map(|(seed, height)| {
            let hash = BlockHash::from_byte_array([seed; 32]);
            self.heights.lock().unwrap().insert(hash, height);
            hash
        });
        self.transactions.lock().unwrap().insert(txid, (tx, hash));
        txid
    }

    fn check(&self) -> Result<(), BitcoinRpcError> {
        if *self.failing.lock().unwrap() {
            return Err("connection reset".into());
        }
        Ok(())
    }
}

impl BitcoinRpc for FakeBitcoinRpc {
    fn get_transaction(&self, txid: &Txid) -> Result<Option<Transaction>, BitcoinRpcError> {
        self.check()?;
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
        self.check()?;
        Ok(self.transactions.lock().unwrap().get(txid).cloned())
    }

    fn get_block_height(&self, block_hash: &BlockHash) -> Result<usize, BitcoinRpcError> {
        self.check()?;
        self.heights
            .lock()
            .unwrap()
            .get(block_hash)
            .copied()
            .ok_or_else(|| "unknown block".into())
    }

    fn network(&self) -> Result<Network, BitcoinRpcError> {
        Ok(self.network)
    }
}

struct Embedded {
    prog: Brc20Prog,
    _db_path: TempDir,
    _exclusive: MutexGuard<'static, ()>,
}

fn open(network: &str, fake: Arc<FakeBitcoinRpc>) -> Embedded {
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

fn witness_tx(seed: u8, items: Vec<Vec<u8>>) -> Transaction {
    Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::new(Txid::from_byte_array([seed; 32]), 0),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::from_slice(&items),
        }],
        output: vec![TxOut {
            value: Amount::from_sat(1_000),
            script_pubkey: ScriptBuf::from_bytes(vec![0x51]),
        }],
    }
}

fn data(txid: &Txid, vin: u64) -> Vec<u8> {
    getTxWitnessCall::new((B256::from_str(&txid.to_string()).unwrap(), U256::from(vin)))
        .abi_encode()
}

async fn eth_call(prog: &Brc20Prog, data: &[u8]) -> Result<String, String> {
    prog.eth_call(
        EthCall::new(
            Some([1u8; 20].into()),
            Some(WITNESS_PRECOMPILE.into()),
            RawBytes::new(format!("0x{}", hex::encode(data))),
        ),
        Some("latest".to_string()),
    )
    .await
    .map_err(|error| error.to_string())
}

async fn call_in_block(
    prog: &Brc20Prog,
    data: &[u8],
    tx_idx: u64,
    inscription_len: u64,
) -> TxReceiptED {
    prog.brc20_call(
        "aaaaabbbbccccddddeeeeffff00001111222233333".to_string(),
        Some(WITNESS_PRECOMPILE.into()),
        None,
        Some(RawBytes::new(format!("0x{}", hex::encode(data)))),
        None,
        12345,
        [0u8; 32].into(),
        tx_idx,
        format!("inscription{tx_idx}"),
        inscription_len,
        [0u8; 32].into(),
    )
    .await
    .unwrap()
    .unwrap()
}

#[tokio::test]
async fn test_returns_stack_height_and_vin_count() {
    let fake = Arc::new(FakeBitcoinRpc::new(Network::Regtest));
    let embedded = open("regtest", fake.clone());
    let txid = fake.add(
        witness_tx(1, vec![vec![1, 2], vec![], vec![0xc0; 33]]),
        Some((1, 4)),
    );
    embedded.prog.brc20_mine(10, 0).await.unwrap();

    let output = eth_call(&embedded.prog, &data(&txid, 0)).await.unwrap();
    let returns = getTxWitnessCall::abi_decode_returns(&hex::decode(output).unwrap()).unwrap();
    assert_eq!(returns.block_height, U256::from(4));
    assert_eq!(returns.vin_count, U256::from(1));
    assert_eq!(
        returns.stack,
        vec![
            Bytes::from(vec![1, 2]),
            Bytes::new(),
            Bytes::from(vec![0xc0; 33])
        ]
    );
}

#[tokio::test]
async fn test_unconfirmed_transaction_fails() {
    let fake = Arc::new(FakeBitcoinRpc::new(Network::Regtest));
    let embedded = open("regtest", fake.clone());
    let txid = fake.add(witness_tx(2, vec![vec![1]]), None);
    embedded.prog.brc20_mine(10, 0).await.unwrap();

    let error = eth_call(&embedded.prog, &data(&txid, 0)).await.unwrap_err();
    assert!(error.contains("PrecompileError"), "{error}");
}

#[tokio::test]
async fn test_unknown_transaction_fails() {
    let fake = Arc::new(FakeBitcoinRpc::new(Network::Regtest));
    let embedded = open("regtest", fake.clone());
    embedded.prog.brc20_mine(10, 0).await.unwrap();
    let txid = witness_tx(3, vec![vec![1]]).compute_txid();

    let error = eth_call(&embedded.prog, &data(&txid, 0)).await.unwrap_err();
    assert!(error.contains("PrecompileError"), "{error}");
}

#[tokio::test]
async fn test_transaction_above_current_height_fails() {
    let fake = Arc::new(FakeBitcoinRpc::new(Network::Regtest));
    let embedded = open("regtest", fake.clone());
    let below = fake.add(witness_tx(4, vec![vec![1]]), Some((4, 5)));
    let above = fake.add(witness_tx(5, vec![vec![1]]), Some((5, 500)));
    embedded.prog.brc20_mine(10, 0).await.unwrap();

    assert!(eth_call(&embedded.prog, &data(&below, 0)).await.is_ok());
    let error = eth_call(&embedded.prog, &data(&above, 0))
        .await
        .unwrap_err();
    assert!(error.contains("PrecompileError"), "{error}");
}

/// Mempools differ between nodes, so these three cases must be indistinguishable to a contract
#[tokio::test]
async fn test_unknown_mempool_and_future_look_identical() {
    let fake = Arc::new(FakeBitcoinRpc::new(Network::Regtest));
    let embedded = open("regtest", fake.clone());
    let unknown = witness_tx(11, vec![vec![1; 2_000]]).compute_txid();
    let mempool = fake.add(witness_tx(12, vec![vec![1; 2_000]]), None);
    let future = fake.add(witness_tx(13, vec![vec![1; 2_000]]), Some((13, 500)));
    embedded.prog.brc20_mine(10, 0).await.unwrap();

    let mut errors = Vec::new();
    for txid in [unknown, mempool, future] {
        errors.push(eth_call(&embedded.prog, &data(&txid, 0)).await.unwrap_err());
    }
    assert_eq!(errors[0], errors[1]);
    assert_eq!(errors[1], errors[2]);

    let mut gas = Vec::new();
    for (tx_idx, txid) in [unknown, mempool, future].iter().enumerate() {
        let receipt = call_in_block(&embedded.prog, &data(txid, 0), tx_idx as u64, 1_000).await;
        let used: u64 = receipt.gas_used.into();
        gas.push(used);
    }
    assert_eq!(gas[0], gas[1]);
    assert_eq!(gas[1], gas[2]);
}

#[tokio::test]
async fn test_gas_follows_transaction_size() {
    let fake = Arc::new(FakeBitcoinRpc::new(Network::Regtest));
    let embedded = open("regtest", fake.clone());
    let small = witness_tx(6, vec![vec![1; 100]]);
    let large = witness_tx(7, vec![vec![1; 5_100]]);
    let extra = (large.total_size() - small.total_size()) as u64;
    let small_id = fake.add(small, Some((6, 1)));
    let large_id = fake.add(large, Some((7, 1)));
    embedded.prog.brc20_mine(10, 0).await.unwrap();

    let first = call_in_block(&embedded.prog, &data(&small_id, 0), 0, 1_000).await;
    let second = call_in_block(&embedded.prog, &data(&large_id, 0), 1, 1_000).await;
    let first_gas: u64 = first.gas_used.into();
    let second_gas: u64 = second.gas_used.into();
    assert!(first_gas > GAS_PER_RPC);
    assert_eq!(second_gas - first_gas, extra * GAS_PER_TX_BYTE);
}

#[tokio::test]
#[should_panic(expected = "Bitcoin RPC unreachable")]
async fn test_failed_lookup_panics_during_block_execution() {
    let fake = Arc::new(FakeBitcoinRpc::new(Network::Regtest));
    let embedded = open("regtest", fake.clone());
    let txid = fake.add(witness_tx(8, vec![vec![1]]), Some((8, 1)));
    embedded.prog.brc20_mine(10, 0).await.unwrap();
    *fake.failing.lock().unwrap() = true;
    call_in_block(&embedded.prog, &data(&txid, 0), 0, 1_000).await;
}

#[tokio::test]
async fn test_failed_lookup_fails_public_call() {
    let fake = Arc::new(FakeBitcoinRpc::new(Network::Regtest));
    let embedded = open("regtest", fake.clone());
    let txid = fake.add(witness_tx(9, vec![vec![1]]), Some((9, 1)));
    embedded.prog.brc20_mine(10, 0).await.unwrap();
    *fake.failing.lock().unwrap() = true;

    let error = eth_call(&embedded.prog, &data(&txid, 0)).await.unwrap_err();
    assert!(error.contains("Bitcoin data unavailable"), "{error}");
}

#[tokio::test]
async fn test_inactive_on_network_without_activation_height() {
    // Signet has a placeholder activation height, so 0xf9 is still an empty account there
    let fake = Arc::new(FakeBitcoinRpc::new(Network::Signet));
    let embedded = open("signet", fake.clone());
    let txid = fake.add(witness_tx(10, vec![vec![1]]), Some((10, 1)));
    embedded.prog.brc20_mine(10, 0).await.unwrap();

    let output = eth_call(&embedded.prog, &data(&txid, 0)).await.unwrap();
    assert!(output.trim_start_matches("0x").is_empty(), "{output}");
}
