use std::error::Error;

use bitcoin::{BlockHash, Network, Transaction, Txid};
use bitcoincore_rpc::jsonrpc::Error::Rpc;
use bitcoincore_rpc::Error::JsonRpc;
use bitcoincore_rpc::{Auth, Client, RpcApi};

/// Error returned by a [`BitcoinRpc`] lookup that could not be answered.
pub type BitcoinRpcError = Box<dyn Error + Send + Sync>;

/// Source of Bitcoin chain data for the Bitcoin precompiles.
///
/// The default implementation talks to a Bitcoin Core node over JSON-RPC, using the
/// `bitcoin_rpc_*` settings in [`Brc20ProgConfig`](crate::Brc20ProgConfig). An indexer that already
/// has this data can pass its own implementation to
/// [`Brc20Prog::open_with_bitcoin_rpc`](crate::Brc20Prog::open_with_bitcoin_rpc) to skip the HTTP
/// round trip.
///
/// Answers feed consensus, so they must be deterministic for a given chain state:
/// * `Ok(None)` means the transaction does not exist, and the precompile call fails.
/// * `Err` means the lookup could not be answered right now. The engine retries it, and panics if
///   it keeps failing, so a transient outage never produces a different execution result.
///
/// Methods are called synchronously from inside EVM execution.
pub trait BitcoinRpc: Send + Sync {
    /// Returns the transaction with `txid`, or `None` if it does not exist.
    fn get_transaction(&self, txid: &Txid) -> Result<Option<Transaction>, BitcoinRpcError>;

    /// Returns the transaction with `txid` and the hash of the block that confirms it (`None` while
    /// unconfirmed), or `None` if the transaction does not exist.
    fn get_transaction_with_block_hash(
        &self,
        txid: &Txid,
    ) -> Result<Option<(Transaction, Option<BlockHash>)>, BitcoinRpcError>;

    /// Returns the height of the block with `block_hash`.
    fn get_block_height(&self, block_hash: &BlockHash) -> Result<usize, BitcoinRpcError>;

    /// Returns the network this source serves, checked against `bitcoin_rpc_network` at startup.
    fn network(&self) -> Result<Network, BitcoinRpcError>;
}

/// [`BitcoinRpc`] backed by a Bitcoin Core node.
pub struct BitcoinCoreRpc {
    client: Client,
}

impl BitcoinCoreRpc {
    pub fn new(url: &str, user: String, password: String) -> Self {
        Self {
            client: Client::new(url, Auth::UserPass(user, password))
                .expect("Failed to create Bitcoin RPC client"),
        }
    }
}

/// Maps error code -5 (RPC_INVALID_ADDRESS_OR_KEY, the txid is unknown) to `None`.
fn not_found_as_none<T>(
    result: Result<T, bitcoincore_rpc::Error>,
) -> Result<Option<T>, BitcoinRpcError> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(JsonRpc(Rpc(ref rpc_error))) if rpc_error.code == -5 => Ok(None),
        Err(error) => Err(error.into()),
    }
}

impl BitcoinRpc for BitcoinCoreRpc {
    fn get_transaction(&self, txid: &Txid) -> Result<Option<Transaction>, BitcoinRpcError> {
        not_found_as_none(self.client.get_raw_transaction(txid, None))
    }

    fn get_transaction_with_block_hash(
        &self,
        txid: &Txid,
    ) -> Result<Option<(Transaction, Option<BlockHash>)>, BitcoinRpcError> {
        let Some(info) = not_found_as_none(self.client.get_raw_transaction_info(txid, None))?
        else {
            return Ok(None);
        };
        // An undecodable transaction fails the precompile call like a missing one
        Ok(info.transaction().ok().map(|tx| (tx, info.blockhash)))
    }

    fn get_block_height(&self, block_hash: &BlockHash) -> Result<usize, BitcoinRpcError> {
        Ok(self.client.get_block_header_info(block_hash)?.height)
    }

    fn network(&self) -> Result<Network, BitcoinRpcError> {
        Ok(self.client.get_blockchain_info()?.chain)
    }
}
