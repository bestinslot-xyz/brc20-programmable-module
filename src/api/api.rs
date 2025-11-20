use std::collections::HashMap;

use alloy::primitives::keccak256;
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;

use crate::api::types::{Base64Bytes, EthCall, GetLogsFilter, PrecompileData};
use crate::db::types::{
    AddressED, BlockResponseED, BytecodeED, LogED, TraceED, TxED, TxReceiptED, B256ED, U256ED,
};
use crate::global::{CARGO_PKG_VERSION, CARGO_RUST_VERSION, CONFIG, INDEXER_ADDRESS};
use crate::types::RawBytes;

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
        "brc20_transact".to_string(),
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
    async fn brc20_version(&self) -> RpcResult<String> {
        Ok(CARGO_PKG_VERSION.to_string())
    }

    /// Mines blocks for the given block count at the timestamp
    #[method(name = "brc20_mine")]
    async fn brc20_mine(&self, block_count: u64, timestamp: u64) -> RpcResult<()>;

    /// Deploys a contract with the given parameters
    #[method(name = "brc20_deploy")]
    async fn brc20_deploy(
        &self,
        from_pkscript: String,
        data: Option<RawBytes>,
        base64_data: Option<Base64Bytes>,
        timestamp: u64,
        hash: B256ED,
        tx_idx: u64,
        inscription_id: String,
        inscription_byte_len: u64,
        op_return_tx_id: B256ED,
    ) -> RpcResult<TxReceiptED>;

    /// Calls a contract with the given parameters
    #[method(name = "brc20_call")]
    async fn brc20_call(
        &self,
        from_pkscript: String,
        contract_address: Option<AddressED>,
        contract_inscription_id: Option<String>,
        data: Option<RawBytes>,
        base64_data: Option<Base64Bytes>,
        timestamp: u64,
        hash: B256ED,
        tx_idx: u64,
        inscription_id: String,
        inscription_byte_len: u64,
        op_return_tx_id: B256ED,
    ) -> RpcResult<Option<TxReceiptED>>;

    /// Calls a contract with the given parameters
    #[method(name = "brc20_transact")]
    async fn brc20_transact(
        &self,
        raw_tx_data: Option<RawBytes>,
        base64_raw_tx_data: Option<Base64Bytes>,
        timestamp: u64,
        hash: B256ED,
        tx_idx: u64,
        inscription_id: String,
        inscription_byte_len: u64,
        op_return_tx_id: B256ED,
    ) -> RpcResult<Vec<TxReceiptED>>;

    /// Deposits brc20 tokens to the given address
    #[method(name = "brc20_deposit")]
    async fn brc20_deposit(
        &self,
        to_pkscript: String,
        ticker: String,
        amount: U256ED,
        timestamp: u64,
        hash: B256ED,
        tx_idx: u64,
        inscription_id: String,
    ) -> RpcResult<TxReceiptED>;

    /// Withdraws brc20 tokens from the given address
    #[method(name = "brc20_withdraw")]
    async fn brc20_withdraw(
        &self,
        from_pkscript: String,
        ticker: String,
        amount: U256ED,
        timestamp: u64,
        hash: B256ED,
        tx_idx: u64,
        inscription_id: String,
    ) -> RpcResult<TxReceiptED>;

    /// Checks BRC20 balance for given address
    #[method(name = "brc20_balance")]
    async fn brc20_balance(&self, pkscript: String, ticker: String) -> RpcResult<String>;

    /// Initialises the BRC20 prog module with the given genesis hash and timestamp
    #[method(name = "brc20_initialise")]
    async fn brc20_initialise(
        &self,
        genesis_hash: B256ED,
        genesis_timestamp: u64,
        genesis_height: u64,
    ) -> RpcResult<()>;

    /// Retrieves transaction receipt for given inscription id
    #[method(name = "brc20_getTxReceiptByInscriptionId")]
    async fn brc20_get_tx_receipt_by_inscription_id(
        &self,
        inscription_id: String,
    ) -> RpcResult<Option<TxReceiptED>>;

    /// Retrieves inscription id for given transaction hash
    #[method(name = "brc20_getInscriptionIdByTxHash")]
    async fn brc20_get_inscription_id_by_tx_hash(
        &self,
        transaction: B256ED,
    ) -> RpcResult<Option<String>>;

    /// Retrieves inscription id by contract address
    #[method(name = "brc20_getInscriptionIdByContractAddress")]
    async fn brc20_get_inscription_id_by_contract_address(
        &self,
        contract_address: AddressED,
    ) -> RpcResult<Option<String>>;

    /// Finalises the block with the given parameters
    #[method(name = "brc20_finaliseBlock")]
    async fn brc20_finalise_block(
        &self,
        timestamp: u64,
        hash: B256ED,
        block_tx_count: u64,
    ) -> RpcResult<()>;

    /// Reverts the state to the given latest valid block number
    #[method(name = "brc20_reorg")]
    async fn brc20_reorg(&self, latest_valid_block_number: u64) -> RpcResult<()>;

    /// Commits the state to the database
    #[method(name = "brc20_commitToDatabase")]
    async fn brc20_commit_to_database(&self) -> RpcResult<()>;

    /// Clears the caches, if used before committing to the database, data will be lost
    #[method(name = "brc20_clearCaches")]
    async fn brc20_clear_caches(&self) -> RpcResult<()>;

    ///
    ///
    /// Eth Methods
    ///
    ///

    /// Returns the latest block number in hex format
    #[method(name = "eth_blockNumber")]
    async fn eth_block_number(&self) -> RpcResult<String>;

    /// Returns the block information for the requested block number
    #[method(name = "eth_getBlockByNumber")]
    async fn eth_get_block_by_number(
        &self,
        block: String,
        is_full: Option<bool>,
    ) -> RpcResult<BlockResponseED>;

    /// Returns the block information for the requested block hash
    #[method(name = "eth_getBlockByHash")]
    async fn eth_get_block_by_hash(
        &self,
        block: B256ED,
        is_full: Option<bool>,
    ) -> RpcResult<BlockResponseED>;

    /// Returns the transaction count by address and block number
    #[method(name = "eth_getTransactionCount")]
    async fn eth_get_transaction_count(
        &self,
        account: AddressED,
        block: String,
    ) -> RpcResult<String>;

    /// Returns the transaction count by block number
    #[method(name = "eth_getBlockTransactionCountByNumber")]
    async fn eth_get_block_transaction_count_by_number(&self, block: String) -> RpcResult<String>;

    /// Returns the transaction count by block hash
    #[method(name = "eth_getBlockTransactionCountByHash")]
    async fn eth_get_block_transaction_count_by_hash(&self, block: B256ED) -> RpcResult<String>;

    /// Gets logs for the given filter
    #[method(name = "eth_getLogs")]
    async fn eth_get_logs(&self, filter: GetLogsFilter) -> RpcResult<Vec<LogED>>;

    /// Calls a contract with the given parameters
    #[method(name = "eth_call")]
    async fn eth_call(&self, eth_call: EthCall, block: Option<String>) -> RpcResult<String>;

    /// Calls a contract with the given parameters for multiple calls
    #[method(name = "eth_callMany")]
    async fn eth_call_many(
        &self,
        eth_calls: Vec<EthCall>,
        block: Option<String>,
        precompile_data: Option<PrecompileData>,
    ) -> RpcResult<Vec<String>>;

    /// Estimates the gas for the given transaction
    #[method(name = "eth_estimateGas")]
    async fn eth_estimate_gas(&self, eth_call: EthCall, block: Option<String>)
        -> RpcResult<String>;

    /// Estimates the gas for the given transactions
    #[method(name = "eth_estimateGasMany")]
    async fn eth_estimate_gas_many(
        &self,
        eth_calls: Vec<EthCall>,
        block: Option<String>,
        precompile_data: Option<PrecompileData>,
    ) -> RpcResult<Vec<String>>;

    /// Get storage for the given contract and memory location
    #[method(name = "eth_getStorageAt")]
    async fn eth_get_storage_at(&self, contract: AddressED, location: U256ED) -> RpcResult<String>;

    /// Returns the bytecode of the contract at the given address
    #[method(name = "eth_getCode")]
    async fn eth_get_code(&self, contract: AddressED) -> RpcResult<BytecodeED>;

    /// Returns the transaction receipt for the given transaction hash
    #[method(name = "eth_getTransactionReceipt")]
    async fn eth_get_transaction_receipt(
        &self,
        transaction: B256ED,
    ) -> RpcResult<Option<TxReceiptED>>;

    /// Returns the trace for the given transaction hash
    #[method(name = "debug_traceTransaction")]
    async fn debug_trace_transaction(&self, transaction: B256ED) -> RpcResult<Option<TraceED>>;

    /// Returns the transaction by hash
    #[method(name = "eth_getTransactionByHash")]
    async fn eth_get_transaction_by_hash(&self, transaction: B256ED) -> RpcResult<Option<TxED>>;

    /// Returns the transaction by block number and index
    #[method(name = "eth_getTransactionByBlockNumberAndIndex")]
    async fn eth_get_transaction_by_block_number_and_index(
        &self,
        number: u64,
        index: Option<u64>,
    ) -> RpcResult<Option<TxED>>;

    /// Returns the transaction by block hash and index
    #[method(name = "eth_getTransactionByBlockHashAndIndex")]
    async fn eth_get_transaction_by_block_hash_and_index(
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
    async fn eth_chain_id(&self) -> RpcResult<String> {
        Ok(format!("0x{:x}", CONFIG.read().chain_id))
    }

    /// Returns max priority fee per gas in hex format (0 in BRC20)
    #[method(name = "eth_maxPriorityFeePerGas")]
    async fn eth_max_priority_fee_per_gas(&self) -> RpcResult<String> {
        Ok("0x0".to_string())
    }

    /// Returns the blob base fee in hex format (0 in BRC20)
    #[method(name = "eth_blobBaseFee")]
    async fn eth_blob_base_fee(&self) -> RpcResult<String> {
        Ok("0x0".to_string())
    }

    /// Returns the balance of the account at the given address (0 in BRC20)
    #[method(name = "eth_getBalance")]
    async fn eth_get_balance(&self, _address: AddressED, _block: String) -> RpcResult<String> {
        Ok("0x0".to_string())
    }

    /// Returns the uncle count of the block at the given block number (0 in BRC20)
    #[method(name = "eth_getUncleCountByBlockNumber")]
    async fn eth_get_uncle_count_by_block_number(&self, _number: u64) -> RpcResult<String> {
        Ok("0x0".to_string())
    }

    /// Returns the uncle count of the block at the given block hash (0 in BRC20)
    #[method(name = "eth_getUncleCountByBlockHash")]
    async fn eth_get_uncle_count_by_block_hash(&self, _hash: B256ED) -> RpcResult<String> {
        Ok("0x0".to_string())
    }

    /// Returns the uncle by block number and index (null in BRC20)
    #[method(name = "eth_getUncleByBlockNumberAndIndex")]
    async fn eth_get_uncle_by_block_number_and_index(
        &self,
        _number: u64,
        _index: u64,
    ) -> RpcResult<Option<String>> {
        Ok(None)
    }

    /// Returns the uncle by block hash and index (null in BRC20)
    #[method(name = "eth_getUncleByBlockHashAndIndex")]
    async fn eth_get_uncle_by_block_hash_and_index(
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

    /// Returns the client version in format brc20_prog/{version}/{os}-{arch}/{rust_version}
    #[method(name = "web3_clientVersion")]
    fn web3_client_version(&self) -> RpcResult<String> {
        Ok(format!(
            "Brc20Prog/v{}/{}-{}/rs{}",
            *CARGO_PKG_VERSION,
            std::env::consts::OS,
            std::env::consts::ARCH,
            *CARGO_RUST_VERSION
        ))
    }

    /// Returns the keccak256 hash of the given bytes
    #[method(name = "web3_sha3")]
    async fn web3_sha3(&self, bytes: RawBytes) -> RpcResult<String> {
        Ok(format!(
            "0x{:x}",
            keccak256(bytes.value().map(|v| v.to_vec()).unwrap_or(vec![]))
        ))
    }

    /// Returns accounts (BRC20 indexer address)
    #[method(name = "eth_accounts")]
    async fn eth_accounts(&self) -> RpcResult<Vec<String>> {
        Ok(vec![INDEXER_ADDRESS.to_string()])
    }

    /// Returns the current gas price in hex format (0 in BRC20, as there's no gas token)
    #[method(name = "eth_gasPrice")]
    async fn eth_gas_price(&self) -> RpcResult<String> {
        Ok("0x0".to_string())
    }

    /// Returns syncing status (false in BRC20)
    #[method(name = "eth_syncing")]
    async fn eth_syncing(&self) -> RpcResult<bool> {
        Ok(false)
    }

    /// Returns txpool content
    #[method(name = "txpool_content")]
    async fn txpool_content(
        &self,
    ) -> RpcResult<HashMap<String, HashMap<AddressED, HashMap<u64, TxED>>>>;

    /// Returns txpool content filtered by from address
    #[method(name = "txpool_contentFrom")]
    async fn txpool_content_from(
        &self,
        from: AddressED,
    ) -> RpcResult<HashMap<String, HashMap<AddressED, HashMap<u64, TxED>>>>;

    /// Returns the raw header for the given block hash or number
    #[method(name = "debug_getRawHeader")]
    async fn debug_get_raw_header(&self, block_hash_or_number: String)
        -> RpcResult<Option<String>>;

    /// Returns the raw block for the given block hash or number
    #[method(name = "debug_getRawBlock")]
    async fn debug_get_raw_block(&self, block_hash_or_number: String) -> RpcResult<Option<String>>;

    /// Returns the raw receipts for the given block hash or number
    #[method(name = "debug_getRawReceipts")]
    async fn debug_get_raw_receipts(
        &self,
        block_hash_or_number: String,
    ) -> RpcResult<Option<Vec<String>>>;
}
