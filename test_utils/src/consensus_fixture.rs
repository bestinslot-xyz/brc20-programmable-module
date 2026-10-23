//! Consensus fixture: drives a fixed workload through the engine and renders everything that
//! feeds the block trace hash, receipts and logs into one text report. The report is pinned with
//! an insta snapshot, so any change in gas accounting, trace shape, log output, tx hashing or
//! signature handling shows up as a snapshot diff. Inputs come from data/consensus_fixture and
//! are pre-encoded, so the workload itself does not depend on the crates under test.

use std::collections::HashMap;
use std::fmt::Write;

use brc20_prog::types::{AddressED, RawBytes, TxReceiptED, U256ED};
use brc20_prog::{Brc20ProgApiClient, Brc20ProgConfig};
use jsonrpsee::http_client::HttpClient;

use crate::{load_file_as_string, spawn_test_server};

/// Mainnet chain id ("BRC20").
pub const CHAIN_ID_MAINNET: u64 = 0x4252433230;
/// Signet/testnet chain id ("BRC20s").
pub const CHAIN_ID_TESTNETS: u64 = 0x425243323073;

const PKSCRIPT: &str = "7465737420706b736372697074"; // "test pkscript"
const TICKER: &str = "FIXT";
const DEFAULT_BYTE_LEN: u64 = 1_000;

/// Which era of a fixture chain a block group belongs to.
pub struct Era {
    pub name: &'static str,
    /// Mine empty blocks until this height before the era starts.
    pub start_height: Option<u64>,
    /// Submit the pre-signed brc20_transact transactions from this file in this era.
    pub transact_file: Option<&'static str>,
}

pub struct FixtureSpec {
    pub network: &'static str,
    pub chain_id: u64,
    pub eras: Vec<Era>,
}

struct Submitted {
    label: String,
    receipt: TxReceiptED,
}

struct Block {
    era: &'static str,
    number: u64,
    txs: Vec<Submitted>,
    rejected: Vec<String>,
}

struct Runner {
    client: HttpClient,
    calls: HashMap<String, String>,
    blocks: Vec<Block>,
    current: Option<Block>,
    tx_idx: u64,
    inscription_counter: u64,
}

fn block_hash(number: u64) -> [u8; 32] {
    let mut hash = [0x5au8; 32];
    hash[24..].copy_from_slice(&number.to_be_bytes());
    hash
}

fn timestamp(number: u64) -> u64 {
    1_700_000_000 + number * 600
}

fn byte_len(label: &str) -> u64 {
    match label {
        "deploy" => 20_000,
        "failures" => 5_000,
        "ethPrecompiles" | "brc20Precompiles" | "bigRuntime" | "bigInitcode" | "calldataSink" => 20_000,
        "burnTop" => 100,
        _ => DEFAULT_BYTE_LEN,
    }
}

fn parse_pairs(file: &str) -> Vec<(String, String)> {
    load_file_as_string(file)
        .expect("fixture file")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let (label, data) = line.split_once(' ').expect("label and data");
            (label.to_string(), data.trim().to_string())
        })
        .collect()
}

fn to_json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).expect("serializable")
}

impl Runner {
    async fn next_block_number(&self) -> u64 {
        let latest = self.client.eth_block_number().await.expect("block number");
        u64::from_str_radix(latest.trim_start_matches("0x"), 16).expect("hex block number") + 1
    }

    async fn begin_block(&mut self, era: &'static str) {
        assert!(self.current.is_none());
        let number = self.next_block_number().await;
        self.tx_idx = 0;
        self.current = Some(Block {
            era,
            number,
            txs: Vec::new(),
            rejected: Vec::new(),
        });
    }

    async fn end_block(&mut self) {
        let block = self.current.take().expect("open block");
        self.client
            .brc20_finalise_block(timestamp(block.number), block_hash(block.number).into(), self.tx_idx)
            .await
            .expect("finalise block");
        self.blocks.push(block);
    }

    fn block_ctx(&mut self) -> (u64, [u8; 32], u64, String) {
        let number = self.current.as_ref().expect("open block").number;
        self.inscription_counter += 1;
        (
            timestamp(number),
            block_hash(number),
            self.tx_idx,
            format!("fixturei{}", self.inscription_counter),
        )
    }

    fn record(&mut self, label: &str, receipts: Vec<TxReceiptED>) {
        let block = self.current.as_mut().expect("open block");
        if receipts.is_empty() {
            block.rejected.push(format!("{label}: no receipt"));
        }
        for (i, receipt) in receipts.into_iter().enumerate() {
            let label = if i == 0 {
                label.to_string()
            } else {
                format!("{label} (queued tx executed after it)")
            };
            block.txs.push(Submitted { label, receipt });
            self.tx_idx += 1;
        }
    }

    fn reject(&mut self, label: &str) {
        let block = self.current.as_mut().expect("open block");
        block.rejected.push(format!("{label}: rpc error"));
    }

    async fn deploy(&mut self, label: &str) -> AddressED {
        let (ts, hash, idx, inscription_id) = self.block_ctx();
        let receipt = self
            .client
            .brc20_deploy(
                PKSCRIPT.to_string(),
                Some(RawBytes::new(load_file_as_string("consensus_fixture/fixture_deploy_tx_data").unwrap())),
                None,
                ts,
                hash.into(),
                idx,
                inscription_id,
                byte_len("deploy"),
                [0x0du8; 32].into(),
            )
            .await
            .expect("deploy");
        let address = receipt.contract_address.clone().expect("deployed contract address");
        self.record(label, vec![receipt]);
        address
    }

    async fn call_raw(&mut self, label: &str, to: Option<AddressED>, inscription: Option<String>, data: String) {
        let (ts, hash, idx, inscription_id) = self.block_ctx();
        let result = self
            .client
            .brc20_call(
                PKSCRIPT.to_string(),
                to,
                inscription,
                Some(RawBytes::new(data)),
                None,
                ts,
                hash.into(),
                idx,
                inscription_id,
                byte_len(label),
                [0x0cu8; 32].into(),
            )
            .await;
        match result {
            Ok(receipt) => self.record(label, receipt.into_iter().collect()),
            Err(_) => self.reject(label),
        }
    }

    async fn call(&mut self, label: &str, to: &AddressED) {
        let data = self.calls.get(label).unwrap_or_else(|| panic!("unknown call {label}")).clone();
        self.call_raw(label, Some(to.clone()), None, data).await;
    }

    async fn transact(&mut self, label: &str, raw_tx: String) {
        let (ts, hash, idx, inscription_id) = self.block_ctx();
        let result = self
            .client
            .brc20_transact(
                Some(RawBytes::new(raw_tx)),
                None,
                ts,
                hash.into(),
                idx,
                inscription_id,
                byte_len(label),
                [0x0eu8; 32].into(),
            )
            .await;
        match result {
            Ok(receipts) => self.record(label, receipts),
            Err(_) => self.reject(label),
        }
    }

    async fn deposit(&mut self, label: &str, value: u64) {
        let (ts, hash, idx, inscription_id) = self.block_ctx();
        let receipt = self
            .client
            .brc20_deposit(PKSCRIPT.to_string(), TICKER.to_string(), amount(value), ts, hash.into(), idx, inscription_id)
            .await
            .expect("deposit");
        self.record(label, vec![receipt]);
    }

    async fn withdraw(&mut self, label: &str, value: u64) {
        let (ts, hash, idx, inscription_id) = self.block_ctx();
        let result = self
            .client
            .brc20_withdraw(PKSCRIPT.to_string(), TICKER.to_string(), amount(value), ts, hash.into(), idx, inscription_id)
            .await;
        match result {
            Ok(receipt) => self.record(label, vec![receipt]),
            Err(_) => self.reject(label),
        }
    }

    async fn run_era(&mut self, era: &Era) {
        if let Some(start) = era.start_height {
            let next = self.next_block_number().await;
            assert!(start >= next, "era {} starts in the past", era.name);
            if start > next {
                self.client
                    .brc20_mine(start - next, timestamp(next))
                    .await
                    .expect("mine");
            }
        }

        // Block 1: deployment, storage, memory, logs, nested frames, creation.
        self.begin_block(era.name).await;
        let fixture = self.deploy("deploy").await;
        let deploy_inscription = format!("fixturei{}", self.inscription_counter);
        for label in ["storageOps", "transientOps", "memoryOps", "logOps", "nested", "callKinds", "createOps"] {
            self.call(label, &fixture).await;
        }
        self.deposit("deposit", 1_000).await;
        self.end_block().await;

        // Block 2: failures, limits, precompiles, environment, BRC20 controller.
        self.begin_block(era.name).await;
        for label in [
            "bigRuntime",
            "bigInitcode",
            "createAndDestroy",
            "destroyLast",
            "createOps",
            "createOpsSecondSalt",
            "failures",
            "revertTop",
            "revertTopString",
            "invalidTop",
            "burnTop",
            "mathOps",
            "envOps",
            "calldataSink",
            "ethPrecompiles",
            "brc20Precompiles",
            "unknownSelector",
        ] {
            self.call(label, &fixture).await;
        }
        // A failing Ethereum precompile call aborts the whole transaction, so edge cases run one each.
        let mut edge_cases: Vec<String> = self
            .calls
            .keys()
            .filter(|label| label.starts_with("precompileEdge:"))
            .cloned()
            .collect();
        edge_cases.sort();
        for label in edge_cases {
            self.call(&label, &fixture).await;
        }
        let storage_ops = self.calls["storageOps"].clone();
        self.call_raw("storageOpsByInscriptionId", None, Some(deploy_inscription), storage_ops)
            .await;
        self.call_raw(
            "callToEmptyAccount",
            Some([0x77u8; 20].into()),
            None,
            self.calls["envOps"].clone(),
        )
        .await;
        self.withdraw("withdraw", 400).await;
        self.withdraw("withdrawMoreThanBalance", 10_000).await;
        self.end_block().await;

        // Block 3: signed transactions (nonce queue, high-s signature, wrong chain id, malformed).
        if let Some(file) = era.transact_file {
            self.begin_block(era.name).await;
            for (label, raw_tx) in parse_pairs(file) {
                self.transact(&format!("transact:{label}"), raw_tx).await;
            }
            self.end_block().await;
        }
    }

    async fn render(&self) -> String {
        let mut out = String::new();
        for block in &self.blocks {
            let tag = format!("0x{:x}", block.number);
            writeln!(out, "======== block {} ({}) ========", block.number, block.era).unwrap();
            let trace_hash = self
                .client
                .debug_get_block_trace_hash(tag.clone())
                .await
                .expect("trace hash");
            writeln!(out, "trace_hash: {}", trace_hash.unwrap_or_default()).unwrap();
            let header = self
                .client
                .eth_get_block_by_number(tag.clone(), Some(false))
                .await
                .expect("block");
            // mineTimestamp is wall-clock processing time, not chain data.
            let mut header = serde_json::to_value(&header).expect("serializable");
            header.as_object_mut().expect("block object").remove("mineTimestamp");
            writeln!(out, "block: {}", to_json(&header)).unwrap();
            let raw_header = self.client.debug_get_raw_header(tag.clone()).await.expect("raw header");
            writeln!(out, "raw_header: {}", raw_header.unwrap_or_default()).unwrap();
            // The raw block lists transactions in storage iteration order, which varies between
            // runs, so only the header and the individual receipt encodings are pinned.
            let mut raw_receipts = self
                .client
                .debug_get_raw_receipts(tag.clone())
                .await
                .expect("raw receipts")
                .unwrap_or_default();
            raw_receipts.sort();
            writeln!(out, "raw_receipts (sorted): {}", to_json(&raw_receipts)).unwrap();
            for rejected in &block.rejected {
                writeln!(out, "rejected: {rejected}").unwrap();
            }
            for tx in &block.txs {
                let hash = tx.receipt.transaction_hash.clone();
                writeln!(out, "-------- tx {} --------", tx.label).unwrap();
                let receipt = self
                    .client
                    .eth_get_transaction_receipt(hash.clone())
                    .await
                    .expect("receipt")
                    .expect("stored receipt");
                writeln!(out, "receipt: {}", to_json(&receipt)).unwrap();
                let trace = self
                    .client
                    .debug_trace_transaction(hash)
                    .await
                    .unwrap_or_else(|e| panic!("trace of {}: {e}", tx.label));
                writeln!(out, "trace: {}", to_json(&trace)).unwrap();
            }
        }
        let balance = self
            .client
            .brc20_balance(PKSCRIPT.to_string(), TICKER.to_string())
            .await
            .expect("balance");
        writeln!(out, "======== final state ========").unwrap();
        writeln!(out, "brc20_balance({TICKER}): {balance}").unwrap();
        out
    }
}

fn amount(value: u64) -> U256ED {
    U256ED::from(value)
}

/// Runs the fixture on a fresh database and returns the rendered report.
pub async fn run_consensus_fixture(spec: FixtureSpec) -> String {
    let (server, client) = spawn_test_server(Brc20ProgConfig {
        bitcoin_rpc_network: spec.network.to_string(),
        chain_id: spec.chain_id,
        ..Default::default()
    })
    .await;

    // brc20_initialise deploys the BRC20 controller and finalises genesis, then pings the
    // Bitcoin RPC, which is unreachable in tests. Tolerate only that failure.
    if let Err(e) = client.brc20_initialise(block_hash(0).into(), timestamp(0), 0).await {
        assert!(e.to_string().contains("Bitcoin RPC"), "brc20_initialise failed: {e}");
    }

    let mut runner = Runner {
        client,
        calls: parse_pairs("consensus_fixture/fixture_calls").into_iter().collect(),
        blocks: Vec::new(),
        current: None,
        tx_idx: 0,
        inscription_counter: 0,
    };
    for era in &spec.eras {
        runner.run_era(era).await;
    }
    let report = runner.render().await;
    server.stop().expect("stop server");
    report
}
