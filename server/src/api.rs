use jsonrpsee::{core::RpcResult, proc_macros::rpc};

use crate::types::{BlockResJSON, SerializableExecutionResult, TxInfo};

#[rpc(server)]
pub trait Brc20ProgApi {
    /// Returns the latest block number
    #[method(name = "custom_blockNumber")]
    async fn block_number(&self) -> RpcResult<u64>;

    /// Returns the block information for the requested block number
    #[method(name = "custom_getBlockByNumber")]
    async fn get_block_by_number(&self, number: u64) -> RpcResult<BlockResJSON>;

    /// Returns the block information for the requested block hash
    #[method(name = "custom_getBlockByHash")]
    async fn get_block_by_hash(&self, hash: String) -> RpcResult<BlockResJSON>;

    /// Mines blocks for the given block count at the timestamp
    #[method(name = "custom_mine")]
    async fn mine(&self, block_cnt: u64, timestamp: u64) -> RpcResult<()>;

    /// Calls a contract with the given parameters
    #[method(name = "custom_call")]
    async fn call(
        &self,
        from: String,
        to: Option<String>,
        data: String,
    ) -> RpcResult<SerializableExecutionResult>;

    /// Adds a transaction to the block
    #[method(name = "custom_addTxToBlock")]
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
    #[method(name = "custom_finaliseBlock")]
    async fn finalise_block(
        &self,
        timestamp: u64,
        hash: String,
        block_tx_cnt: u64,
    ) -> RpcResult<()>;

    /// Finalises the block with the given parameters and transactions
    #[method(name = "custom_finaliseBlockWithTxes")]
    async fn finalise_block_with_txes(
        &self,
        timestamp: u64,
        hash: String,
        txes: Vec<TxInfo>,
    ) -> RpcResult<Vec<SerializableExecutionResult>>;

    /// Reverts the state to the given latest valid block number
    #[method(name = "custom_reorg")]
    async fn reorg(&self, latest_valid_block_number: u64) -> RpcResult<()>;

    /// Commits the state to the database
    #[method(name = "custom_commitToDatabase")]
    async fn commit_to_database(&self) -> RpcResult<()>;

    /// Clears the caches, if used before committing to the database, data will be lost
    #[method(name = "custom_clearCaches")]
    async fn clear_caches(&self) -> RpcResult<()>;

    /// Returns the bytecode of the contract at the given address
    #[method(name = "custom_getCode")]
    async fn get_code(&self, address: String) -> RpcResult<String>;
}
