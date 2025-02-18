use db::types::{BlockResponseED, LogResponseED, TxED, TxReceiptED};
use jsonrpsee::{core::RpcResult, proc_macros::rpc};

use crate::types::TxInfo;

#[rpc(server)]
pub trait Brc20ProgApi {
    ///
    ///
    /// BRC20 Methods, these methods are intended for the indexers
    /// TODO: Authentication!
    ///
    ///

    /// Mines blocks for the given block count at the timestamp
    #[method(name = "brc20_mine")]
    async fn mine(&self, block_cnt: u64, timestamp: u64) -> RpcResult<()>;

    /// Adds a transaction to the block
    #[method(name = "brc20_addTxToBlock")]
    async fn add_tx_to_block(
        &self,
        from: String,
        to: Option<String>,
        data: String,
        timestamp: u64,
        hash: String,
        tx_idx: u64,
    ) -> RpcResult<TxReceiptED>;

    /// Finalises the block with the given parameters
    #[method(name = "brc20_finaliseBlock")]
    async fn finalise_block(
        &self,
        timestamp: u64,
        hash: String,
        block_tx_cnt: u64,
    ) -> RpcResult<()>;

    /// Finalises the block with the given parameters and transactions
    #[method(name = "brc20_finaliseBlockWithTxes")]
    async fn finalise_block_with_txes(
        &self,
        timestamp: u64,
        hash: String,
        txes: Vec<TxInfo>,
    ) -> RpcResult<Vec<TxReceiptED>>;

    /// Reverts the state to the given latest valid block number
    #[method(name = "brc20_reorg")]
    async fn reorg(&self, latest_valid_block_number: u64) -> RpcResult<()>;

    /// Commits the state to the database
    #[method(name = "brc20_commitToDatabase")]
    async fn commit_to_database(&self) -> RpcResult<()>;

    /// Clears the caches, if used before committing to the database, data will be lost
    #[method(name = "brc20_clearCaches")]
    async fn clear_caches(&self) -> RpcResult<()>;

    ///
    ///
    /// Eth Methods
    ///
    ///

    /// Returns the latest block number in hex format
    #[method(name = "eth_blockNumber")]
    async fn block_number(&self) -> RpcResult<String>;

    /// Returns the block information for the requested block number
    #[method(name = "eth_getBlockByNumber")]
    async fn get_block_by_number(&self, block: String) -> RpcResult<BlockResponseED>;

    /// Returns the block information for the requested block hash
    #[method(name = "eth_getBlockByHash")]
    async fn get_block_by_hash(&self, block: String) -> RpcResult<BlockResponseED>;

    /// Returns the transaction count by address and block number
    #[method(name = "eth_getTransactionCount")]
    async fn get_transaction_count(&self, account: String, block: String) -> RpcResult<String>;

    /// Returns the transaction count by block number
    #[method(name = "eth_getBlockTransactionCountByNumber")]
    async fn get_block_transaction_count_by_number(&self, block: String) -> RpcResult<String>;

    /// Returns the transaction count by block hash
    #[method(name = "eth_getBlockTransactionCountByHash")]
    async fn get_block_transaction_count_by_hash(&self, block: String) -> RpcResult<String>;

    /// Gets logs for the given filter
    #[method(name = "eth_getLogs")]
    async fn get_logs(&self, filter: GetLogsFilter) -> RpcResult<Vec<LogResponseED>>;

    /// Calls a contract with the given parameters
    #[method(name = "eth_call")]
    async fn call(&self, from: String, to: Option<String>, data: String) -> RpcResult<TxReceiptED>;

    /// Estimates the gas for the given transaction
    #[method(name = "eth_estimateGas")]
    async fn estimate_gas(
        &self,
        from: String,
        to: Option<String>,
        data: String,
    ) -> RpcResult<String>;

    /// Get storage for the given contract and memory location
    #[method(name = "eth_getStorageAt")]
    async fn get_storage_at(&self, contract: String, location: String) -> RpcResult<String>;

    /// Returns the bytecode of the contract at the given address
    #[method(name = "eth_getCode")]
    async fn get_code(&self, contract: String) -> RpcResult<String>;

    /// Returns the transaction receipt for the given transaction hash
    #[method(name = "eth_getTransactionReceipt")]
    async fn get_transaction_receipt(&self, transaction: String) -> RpcResult<Option<TxReceiptED>>;

    /// Returns the transaction by hash
    #[method(name = "eth_getTransactionByHash")]
    async fn get_transaction_by_hash(&self, transaction: String) -> RpcResult<Option<TxED>>;

    /// Returns the transaction by block number and index
    #[method(name = "eth_getTransactionByBlockNumberAndIndex")]
    async fn get_transaction_by_block_number_and_index(
        &self,
        number: u64,
        index: u64,
    ) -> RpcResult<Option<TxED>>;

    /// Returns the transaction by block hash and index
    #[method(name = "eth_getTransactionByBlockHashAndIndex")]
    async fn get_transaction_by_block_hash_and_index(
        &self,
        hash: String,
        index: u64,
    ) -> RpcResult<Option<TxED>>;

    ///
    ///
    /// Eth methods with static values
    ///
    ///

    /// Returns the chain id in hex format ("BRC20" in hex)
    #[method(name = "eth_chainId")]
    async fn chain_id(&self) -> RpcResult<String> {
        Ok("0x4252433230".to_string())
    }

    /// Returns max priority fee per gas in hex format (0 in BRC20)
    #[method(name = "eth_maxPriorityFeePerGas")]
    async fn max_priority_fee_per_gas(&self) -> RpcResult<String> {
        Ok("0x0".to_string())
    }

    /// Returns the blob base fee in hex format (0 in BRC20)
    #[method(name = "eth_blobBaseFee")]
    async fn base_fee_per_gas(&self) -> RpcResult<String> {
        Ok("0x0".to_string())
    }

    /// Returns the balance of the account at the given address (0 in BRC20)
    #[method(name = "eth_getBalance")]
    async fn get_balance(&self, _address: String) -> RpcResult<String> {
        Ok("0x0".to_string())
    }

    /// Returns the uncle count of the block at the given block number (0 in BRC20)
    #[method(name = "eth_getUncleCountByBlockNumber")]
    async fn get_uncle_count_by_block_number(&self, _number: u64) -> RpcResult<String> {
        Ok("0x0".to_string())
    }

    /// Returns the uncle count of the block at the given block hash (0 in BRC20)
    #[method(name = "eth_getUncleCountByBlockHash")]
    async fn get_uncle_count_by_block_hash(&self, _hash: String) -> RpcResult<String> {
        Ok("0x0".to_string())
    }

    /// Returns the uncle by block number and index (null in BRC20)
    #[method(name = "eth_getUncleByBlockNumberAndIndex")]
    async fn get_uncle_by_block_number_and_index(
        &self,
        _number: u64,
        _index: u64,
    ) -> RpcResult<Option<String>> {
        Ok(None)
    }

    /// Returns the uncle by block hash and index (null in BRC20)
    #[method(name = "eth_getUncleByBlockHashAndIndex")]
    async fn get_uncle_by_block_hash_and_index(
        &self,
        _hash: String,
        _index: u64,
    ) -> RpcResult<Option<String>> {
        Ok(None)
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct GetLogsFilter {
    #[serde(rename = "fromBlock")]
    pub from_block: Option<String>,
    #[serde(rename = "toBlock")]
    pub to_block: Option<String>,
    pub address: Option<String>,
    pub topics: Option<Vec<String>>,
}
