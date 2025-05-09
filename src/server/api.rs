use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;

use crate::db::types::{
    AddressED, BlockResponseED, BytecodeED, LogED, TraceED, TxED, TxReceiptED, B256ED, U256ED,
};
use crate::global::{CARGO_PKG_VERSION, CHAIN_ID_STRING, INDEXER_ADDRESS};
use crate::server::types::{EncodedBytes, EthCall, GetLogsFilter};

lazy_static::lazy_static! {
    // BRC20 Methods intended for the indexers, so they require auth
    pub static ref INDEXER_METHODS: Vec<String> = vec![
        "brc20_mine".to_string(),
        "brc20_deploy".to_string(),
        "brc20_call".to_string(),
        "brc20_deposit".to_string(),
        "brc20_withdraw".to_string(),
        "brc20_initialise".to_string(),
        "brc20_finaliseBlock".to_string(),
        "brc20_reorg".to_string(),
        "brc20_commitToDatabase".to_string(),
        "brc20_clearCaches".to_string(),
    ];
}

#[rpc(server, client)]
pub trait Brc20ProgApi {
    /// BRC20 Methods, these methods are intended for the indexers

    /// Returns current brc20_prog version
    #[method(name = "brc20_version")]
    async fn version(&self) -> RpcResult<String> {
        Ok(CARGO_PKG_VERSION.to_string())
    }

    /// Mines blocks for the given block count at the timestamp
    #[method(name = "brc20_mine")]
    async fn mine(&self, block_count: u64, timestamp: u64) -> RpcResult<()>;

    /// Deploys a contract with the given parameters
    #[method(name = "brc20_deploy")]
    async fn deploy_contract(
        &self,
        from_pkscript: String,
        data: EncodedBytes,
        timestamp: u64,
        hash: B256ED,
        tx_idx: u64,
        inscription_id: Option<String>,
        inscription_byte_len: Option<u64>,
    ) -> RpcResult<TxReceiptED>;

    /// Calls a contract with the given parameters
    #[method(name = "brc20_call")]
    async fn call_contract(
        &self,
        from_pkscript: String,
        contract_address: Option<AddressED>,
        contract_inscription_id: Option<String>,
        data: EncodedBytes,
        timestamp: u64,
        hash: B256ED,
        tx_idx: u64,
        inscription_id: Option<String>,
        inscription_byte_len: Option<u64>,
    ) -> RpcResult<Option<TxReceiptED>>;

    /// Deposits brc20 tokens to the given address
    #[method(name = "brc20_deposit")]
    async fn deposit(
        &self,
        to_pkscript: String,
        ticker: String,
        amount: U256ED,
        timestamp: u64,
        hash: B256ED,
        tx_idx: u64,
        inscription_id: Option<String>,
    ) -> RpcResult<TxReceiptED>;

    /// Withdraws brc20 tokens from the given address
    #[method(name = "brc20_withdraw")]
    async fn withdraw(
        &self,
        from_pkscript: String,
        ticker: String,
        amount: U256ED,
        timestamp: u64,
        hash: B256ED,
        tx_idx: u64,
        inscription_id: Option<String>,
    ) -> RpcResult<TxReceiptED>;

    /// Checks BRC20 balance for given address
    #[method(name = "brc20_balance")]
    async fn balance(&self, pkscript: String, ticker: String) -> RpcResult<String>;

    /// Initialises the BRC20 prog module with the given genesis hash and timestamp
    #[method(name = "brc20_initialise")]
    async fn initialise(
        &self,
        genesis_hash: B256ED,
        genesis_timestamp: u64,
        genesis_height: u64,
    ) -> RpcResult<()>;

    /// Retrieves transaction receipt for given inscription id
    #[method(name = "brc20_getTxReceiptByInscriptionId")]
    async fn get_transaction_receipt_by_inscription_id(
        &self,
        inscription_id: String,
    ) -> RpcResult<Option<TxReceiptED>>;

    /// Retrieves inscription id for given transaction hash
    #[method(name = "brc20_getInscriptionIdByTxHash")]
    async fn get_inscription_id_by_tx_hash(&self, transaction: B256ED)
        -> RpcResult<Option<String>>;

    /// Retrieves inscription id by contract address
    #[method(name = "brc20_getInscriptionIdByContractAddress")]
    async fn get_inscription_id_by_contract_address(
        &self,
        contract_address: AddressED,
    ) -> RpcResult<Option<String>>;

    /// Finalises the block with the given parameters
    #[method(name = "brc20_finaliseBlock")]
    async fn finalise_block(
        &self,
        timestamp: u64,
        hash: B256ED,
        block_tx_count: u64,
    ) -> RpcResult<()>;

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
    async fn get_block_by_number(
        &self,
        block: String,
        is_full: Option<bool>,
    ) -> RpcResult<BlockResponseED>;

    /// Returns the block information for the requested block hash
    #[method(name = "eth_getBlockByHash")]
    async fn get_block_by_hash(
        &self,
        block: B256ED,
        is_full: Option<bool>,
    ) -> RpcResult<BlockResponseED>;

    /// Returns the transaction count by address and block number
    #[method(name = "eth_getTransactionCount")]
    async fn get_transaction_count(&self, account: AddressED, block: String) -> RpcResult<String>;

    /// Returns the transaction count by block number
    #[method(name = "eth_getBlockTransactionCountByNumber")]
    async fn get_block_transaction_count_by_number(&self, block: String) -> RpcResult<String>;

    /// Returns the transaction count by block hash
    #[method(name = "eth_getBlockTransactionCountByHash")]
    async fn get_block_transaction_count_by_hash(&self, block: B256ED) -> RpcResult<String>;

    /// Gets logs for the given filter
    #[method(name = "eth_getLogs")]
    async fn get_logs(&self, filter: GetLogsFilter) -> RpcResult<Vec<LogED>>;

    /// Calls a contract with the given parameters
    #[method(name = "eth_call")]
    async fn call(&self, eth_call: EthCall, block: Option<String>) -> RpcResult<String>;

    /// Estimates the gas for the given transaction
    #[method(name = "eth_estimateGas")]
    async fn estimate_gas(&self, eth_call: EthCall, block: Option<String>) -> RpcResult<String>;

    /// Get storage for the given contract and memory location
    #[method(name = "eth_getStorageAt")]
    async fn get_storage_at(&self, contract: AddressED, location: U256ED) -> RpcResult<String>;

    /// Returns the bytecode of the contract at the given address
    #[method(name = "eth_getCode")]
    async fn get_code(&self, contract: AddressED) -> RpcResult<BytecodeED>;

    /// Returns the transaction receipt for the given transaction hash
    #[method(name = "eth_getTransactionReceipt")]
    async fn get_transaction_receipt(&self, transaction: B256ED) -> RpcResult<Option<TxReceiptED>>;

    /// Returns the trace for the given transaction hash
    #[method(name = "debug_traceTransaction")]
    async fn debug_trace_transaction(&self, transaction: B256ED) -> RpcResult<Option<TraceED>>;

    /// Returns the transaction by hash
    #[method(name = "eth_getTransactionByHash")]
    async fn get_transaction_by_hash(&self, transaction: B256ED) -> RpcResult<Option<TxED>>;

    /// Returns the transaction by block number and index
    #[method(name = "eth_getTransactionByBlockNumberAndIndex")]
    async fn get_transaction_by_block_number_and_index(
        &self,
        number: u64,
        index: Option<u64>,
    ) -> RpcResult<Option<TxED>>;

    /// Returns the transaction by block hash and index
    #[method(name = "eth_getTransactionByBlockHashAndIndex")]
    async fn get_transaction_by_block_hash_and_index(
        &self,
        hash: B256ED,
        index: Option<u64>,
    ) -> RpcResult<Option<TxED>>;

    ///
    ///
    /// Eth methods with static values
    ///
    ///

    /// Returns the chain id in hex format ("BRC20" in hex)
    #[method(name = "eth_chainId")]
    async fn chain_id(&self) -> RpcResult<String> {
        Ok(CHAIN_ID_STRING.to_string())
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
    async fn get_balance(&self, _address: AddressED, _block: String) -> RpcResult<String> {
        Ok("0x0".to_string())
    }

    /// Returns the uncle count of the block at the given block number (0 in BRC20)
    #[method(name = "eth_getUncleCountByBlockNumber")]
    async fn get_uncle_count_by_block_number(&self, _number: u64) -> RpcResult<String> {
        Ok("0x0".to_string())
    }

    /// Returns the uncle count of the block at the given block hash (0 in BRC20)
    #[method(name = "eth_getUncleCountByBlockHash")]
    async fn get_uncle_count_by_block_hash(&self, _hash: B256ED) -> RpcResult<String> {
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
        _hash: B256ED,
        _index: u64,
    ) -> RpcResult<Option<String>> {
        Ok(None)
    }

    /// Returns net version
    #[method(name = "net_version")]
    async fn net_version(&self) -> RpcResult<String> {
        Ok("4252433230".to_string())
    }

    /// Returns accounts (BRC20 indexer address)
    #[method(name = "eth_accounts")]
    async fn accounts(&self) -> RpcResult<Vec<String>> {
        Ok(vec![INDEXER_ADDRESS.to_string()])
    }

    /// Returns the current gas price in hex format (0 in BRC20, as there's no gas token)
    #[method(name = "eth_gasPrice")]
    async fn gas_price(&self) -> RpcResult<String> {
        Ok("0x0".to_string())
    }

    /// Returns syncing status (false in BRC20)
    #[method(name = "eth_syncing")]
    async fn syncing(&self) -> RpcResult<bool> {
        Ok(false)
    }
}
