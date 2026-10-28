use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use alloy::primitives::B256;
use bitcoin::{BlockHash, KnownHrp, Network, Transaction, Txid};
use revm::primitives::Bytes;

use crate::engine::precompiles::bitcoin_rpc::{BitcoinCoreRpc, BitcoinRpc, BitcoinRpcError};
use crate::global::{SharedData, CONFIG};

lazy_static::lazy_static! {
    static ref BITCOIN_RPC_URL: SharedData<String> = SharedData::new(Default::default());
    static ref BITCOIN_RPC_USER: SharedData<String> = SharedData::new(Default::default());
    static ref BITCOIN_RPC_PASSWORD: SharedData<String> = SharedData::new(Default::default());
    static ref BTC_CLIENT: SharedData<Arc<BitcoinCoreRpc>> = SharedData::new(Arc::new(new_core_client()));

    /// Caller-supplied source that replaces BTC_CLIENT while set
    static ref INJECTED_BITCOIN_RPC: SharedData<Option<Arc<dyn BitcoinRpc>>> = SharedData::new(None);

    static ref BLOCK_HASH_FUTURE_PLACEHOLDER: BlockHash = BlockHash::from_str("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();
}

const BITCOIN_RPC_RETRIES: u32 = 5;

fn new_core_client() -> BitcoinCoreRpc {
    let config = CONFIG.read();
    BitcoinCoreRpc::new(
        &config.bitcoin_rpc_url,
        config.bitcoin_rpc_user.clone(),
        config.bitcoin_rpc_password.clone(),
    )
}

/// Makes `rpc` the source of Bitcoin data for this process, or restores the Bitcoin Core client
/// built from the config when `None`.
pub fn set_bitcoin_rpc(rpc: Option<Arc<dyn BitcoinRpc>>) {
    INJECTED_BITCOIN_RPC.write_fn_unchecked(|value| *value = rpc);
}

fn bitcoin_rpc() -> Arc<dyn BitcoinRpc> {
    if let Some(rpc) = INJECTED_BITCOIN_RPC.read().as_ref() {
        return rpc.clone();
    }
    BTC_CLIENT.read().clone()
}

pub fn update_bitcoin_client() {
    // if config has changed, create a new client
    if !BITCOIN_RPC_URL.read().eq(&CONFIG.read().bitcoin_rpc_url)
        || !BITCOIN_RPC_USER.read().eq(&CONFIG.read().bitcoin_rpc_user)
        || !BITCOIN_RPC_PASSWORD
            .read()
            .eq(&CONFIG.read().bitcoin_rpc_password)
    {
        BITCOIN_RPC_URL.write_fn_unchecked(|url| {
            *url = CONFIG.read().bitcoin_rpc_url.clone();
        });
        BITCOIN_RPC_USER.write_fn_unchecked(|user| {
            *user = CONFIG.read().bitcoin_rpc_user.clone();
        });
        BITCOIN_RPC_PASSWORD.write_fn_unchecked(|password| {
            *password = CONFIG.read().bitcoin_rpc_password.clone();
        });
        BTC_CLIENT.write_fn_unchecked(|client| {
            *client = Arc::new(new_core_client());
        });
    }
}

pub fn get_bitcoin_network() -> Network {
    match CONFIG.read().bitcoin_rpc_network.as_str() {
        "bitcoin" => Network::Bitcoin,
        "mainnet" => Network::Bitcoin,
        "signet" => Network::Signet,
        "testnet" => Network::Testnet,
        "testnet4" => Network::Testnet4,
        "regtest" => Network::Regtest,
        _ => Network::Testnet4,
    }
}

pub fn get_bitcoin_hrp() -> KnownHrp {
    match get_bitcoin_network() {
        Network::Bitcoin => KnownHrp::Mainnet,
        Network::Testnet => KnownHrp::Testnets,
        Network::Testnet4 => KnownHrp::Testnets,
        Network::Signet => KnownHrp::Testnets,
        Network::Regtest => KnownHrp::Regtest,
    }
}

pub fn validate_bitcoin_rpc_status() -> Result<(), Box<dyn Error>> {
    let injected = INJECTED_BITCOIN_RPC.read().clone();
    if injected.is_none() && CONFIG.read().bitcoin_rpc_url.is_empty() {
        return Err("Please configure BITCOIN_RPC_URL".into());
    }
    if CONFIG.read().bitcoin_rpc_network.is_empty() {
        return Err("Please configure BITCOIN_RPC_NETWORK".into());
    }

    let rpc = match injected {
        Some(rpc) => rpc,
        None => {
            // Update the client if the config has changed
            update_bitcoin_client();
            bitcoin_rpc()
        }
    };

    let Ok(network) = rpc.network() else {
        return Err("Bitcoin RPC unreachable.".into());
    };

    if network != get_bitcoin_network() {
        return Err(format!(
            "Bitcoin RPC network mismatch. Expected: {:?}, got: {:?}",
            get_bitcoin_network(),
            network
        )
        .into());
    }

    Ok(())
}

/// Where a Bitcoin lookup runs, which decides what a [`BitcoinRpc`] error does.
///
/// Each entry point that builds an EVM picks the mode and the precompiles carry it in
/// [`PrecompileCall`](crate::engine::precompiles::PrecompileCall). It lives in the EVM value
/// rather than in process or thread state, so one call's mode never reaches another call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BitcoinLookup {
    /// Block execution, the consensus path. A lookup error is retried and then panics, so an
    /// unreachable source stops the node instead of changing an execution result.
    BlockExecution,
    /// A public read such as `eth_call` or `eth_estimateGas`. A lookup error fails the call after
    /// one attempt, so an unauthenticated caller can neither crash the node nor stall it.
    PublicCall,
}

/// Message of the error that aborts a public call whose Bitcoin lookup failed.
pub const BITCOIN_DATA_UNAVAILABLE: &str = "Bitcoin data unavailable";

/// A [`BitcoinRpc`] lookup failed during a public call.
#[derive(Debug, PartialEq, Eq)]
pub struct BitcoinDataUnavailable;

impl std::fmt::Display for BitcoinDataUnavailable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(BITCOIN_DATA_UNAVAILABLE)
    }
}

impl Error for BitcoinDataUnavailable {}

/// Runs `lookup` against the current source.
///
/// During block execution a failed lookup is retried, and the node panics once the retries run
/// out, so an unreachable source never changes an execution result. During a public call a failed
/// lookup returns [`BitcoinDataUnavailable`] at once.
fn run_lookup<T>(
    mode: BitcoinLookup,
    lookup: impl Fn(&dyn BitcoinRpc) -> Result<T, BitcoinRpcError>,
) -> Result<T, BitcoinDataUnavailable> {
    let rpc = bitcoin_rpc();
    let mut retries_left = BITCOIN_RPC_RETRIES;
    loop {
        match (lookup(rpc.as_ref()), mode) {
            (Ok(value), _) => return Ok(value),
            (Err(_), BitcoinLookup::PublicCall) => return Err(BitcoinDataUnavailable),
            (Err(_), BitcoinLookup::BlockExecution) if retries_left > 0 => {
                retries_left -= 1;
                sleep(Duration::from_secs(1));
            }
            (Err(error), BitcoinLookup::BlockExecution) => {
                panic!("Bitcoin RPC unreachable. Response: {:?}", error)
            }
        }
    }
}

fn parse_txid(txid: &B256) -> Option<Txid> {
    Txid::from_str(&hex::encode(txid.as_slice())).ok()
}

/// Returns the transaction with `txid` and the hash of its block, from `btc_tx_hexes` or the
/// Bitcoin source. `Ok(None)` means the transaction is missing or undecodable, which fails the
/// precompile call.
pub fn get_transaction_and_block_hash_with_overrides(
    txid: &B256,
    btc_tx_hexes: &HashMap<B256, Bytes>,
    mode: BitcoinLookup,
) -> Result<Option<(Transaction, Option<BlockHash>)>, BitcoinDataUnavailable> {
    if let Some(tx_hex) = btc_tx_hexes.get(txid) {
        // An override carries no block, so it gets the placeholder hash at height 0
        return Ok(bitcoin::consensus::deserialize(tx_hex)
            .ok()
            .map(|tx| (tx, Some(*BLOCK_HASH_FUTURE_PLACEHOLDER))));
    }
    let Some(bitcoin_txid) = parse_txid(txid) else {
        return Ok(None);
    };
    run_lookup(mode, |rpc| {
        rpc.get_transaction_with_block_hash(&bitcoin_txid)
    })
}

/// Returns the transaction with `txid`, from `btc_tx_hexes` or the Bitcoin source. `Ok(None)`
/// means the transaction is missing or undecodable, which fails the precompile call.
pub fn get_transaction_with_overrides(
    txid: &B256,
    btc_tx_hexes: &HashMap<B256, Bytes>,
    mode: BitcoinLookup,
) -> Result<Option<Transaction>, BitcoinDataUnavailable> {
    if let Some(tx_hex) = btc_tx_hexes.get(txid) {
        return Ok(bitcoin::consensus::deserialize(tx_hex).ok());
    }
    let Some(bitcoin_txid) = parse_txid(txid) else {
        return Ok(None);
    };
    run_lookup(mode, |rpc| rpc.get_transaction(&bitcoin_txid))
}

/// Returns the height of the block with `block_hash`.
pub fn get_block_height(
    block_hash: &BlockHash,
    mode: BitcoinLookup,
) -> Result<usize, BitcoinDataUnavailable> {
    if block_hash == &*BLOCK_HASH_FUTURE_PLACEHOLDER {
        return Ok(0);
    }
    run_lookup(mode, |rpc| rpc.get_block_height(block_hash))
}

#[cfg(test)]
mod tests {
    use alloy::primitives::hex::FromHex;
    use alloy::primitives::FixedBytes;

    use super::*;

    #[test]
    fn test_get_raw_transaction() {
        if validate_bitcoin_rpc_status().is_err() {
            return;
        }

        let txid_string = "4183fb733b9553ca8b93208c91dda18bee3d0b8510720b15d76d979af7fd9926";
        let response = get_transaction_and_block_hash_with_overrides(
            &FixedBytes::from_hex(txid_string).unwrap(),
            &HashMap::new(),
            BitcoinLookup::BlockExecution,
        );
        assert_eq!(
            response.unwrap().unwrap().0.compute_txid().to_string(),
            txid_string
        );
    }
}
