use jsonrpsee::{core::RpcResult, proc_macros::rpc};

use crate::types::{BlockResJSON, SerializableExecutionResult, TxInfo};

#[rpc(server)]
pub trait Brc20ProgApi {
    /// Returns the chain id in hex format ("BRC20" in hex)
    #[method(name = "eth_chainId")]
    async fn chain_id(&self) -> RpcResult<String> {
        Ok("0x4252433230".to_string())
    }

    /// Returns the latest block number in hex format
    #[method(name = "eth_blockNumber")]
    async fn block_number(&self) -> RpcResult<String>;

    /// Returns the block information for the requested block number
    #[method(name = "eth_getBlockByNumber")]
    async fn get_block_by_number(&self, number: String) -> RpcResult<BlockResJSON>;

    /// Returns the block information for the requested block hash
    #[method(name = "eth_getBlockByHash")]
    async fn get_block_by_hash(&self, hash: String) -> RpcResult<BlockResJSON>;

    /// Mines blocks for the given block count at the timestamp
    #[method(name = "eth_mine")]
    async fn mine(&self, block_cnt: u64, timestamp: u64) -> RpcResult<()>;

    /// Calls a contract with the given parameters
    #[method(name = "eth_call")]
    async fn call(
        &self,
        from: String,
        to: Option<String>,
        data: String,
    ) -> RpcResult<SerializableExecutionResult>;

    /// Adds a transaction to the block
    #[method(name = "eth_addTxToBlock")]
    async fn add_tx_to_block(
        &self,
        from: String,
        to: Option<String>,
        data: String,
        timestamp: u64,
        hash: String,
        tx_idx: u64,
    ) -> RpcResult<SerializableExecutionResult>;

    /// Finalises the block with the given parameters
    #[method(name = "eth_finaliseBlock")]
    async fn finalise_block(
        &self,
        timestamp: u64,
        hash: String,
        block_tx_cnt: u64,
    ) -> RpcResult<()>;

    /// Finalises the block with the given parameters and transactions
    #[method(name = "eth_finaliseBlockWithTxes")]
    async fn finalise_block_with_txes(
        &self,
        timestamp: u64,
        hash: String,
        txes: Vec<TxInfo>,
    ) -> RpcResult<Vec<SerializableExecutionResult>>;

    /// Reverts the state to the given latest valid block number
    #[method(name = "eth_reorg")]
    async fn reorg(&self, latest_valid_block_number: u64) -> RpcResult<()>;

    /// Commits the state to the database
    #[method(name = "eth_commitToDatabase")]
    async fn commit_to_database(&self) -> RpcResult<()>;

    /// Clears the caches, if used before committing to the database, data will be lost
    #[method(name = "eth_clearCaches")]
    async fn clear_caches(&self) -> RpcResult<()>;

    /// Returns the bytecode of the contract at the given address
    #[method(name = "eth_getCode")]
    async fn get_code(&self, address: String) -> RpcResult<String>;

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
