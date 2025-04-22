use std::str::FromStr;

use alloy_primitives::hex::FromHex;
use alloy_primitives::{Address, Bytes, FixedBytes, B256, U256};
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use serde::Deserialize;

use crate::config::BRC20_PROG_CONFIG;
use crate::db::types::{BlockResponseED, BytecodeED, LogED, TraceED, TxED, TxReceiptED};

lazy_static::lazy_static! {
    pub static ref CHAIN_ID: u64 = 0x4252433230;
    pub static ref CHAIN_ID_STRING: String = CHAIN_ID.to_string();
    pub static ref INDEXER_ADDRESS: Address = "0x0000000000000000000000000000000000003Ca6".parse().expect("Failed to parse indexer address");
    pub static ref INDEXER_ADDRESS_STRING: String = INDEXER_ADDRESS.to_string();
    pub static ref INVALID_ADDRESS: Address = "0x000000000000000000000000000000000000dead".parse().expect("Failed to parse invalid address");

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

#[rpc(server)]
pub trait Brc20ProgApi {
    ///
    ///
    /// BRC20 Methods, these methods are intended for the indexers
    /// TODO: Authentication!
    ///
    ///

    /// Returns current brc20_prog_version
    #[method(name = "brc20_version")]
    async fn version(&self) -> RpcResult<String> {
        Ok(BRC20_PROG_CONFIG.pkg_version.clone())
    }

    /// Mines blocks for the given block count at the timestamp
    #[method(name = "brc20_mine")]
    async fn mine(&self, block_count: u64, timestamp: u64) -> RpcResult<()>;

    #[method(name = "brc20_deploy")]
    async fn deploy_contract(
        &self,
        from_pkscript: String,
        data: BytesWrapper,
        timestamp: u64,
        hash: B256Wrapper,
        tx_idx: u64,
        inscription_id: Option<String>,
        inscription_byte_len: Option<u64>,
    ) -> RpcResult<TxReceiptED>;

    #[method(name = "brc20_call")]
    async fn call_contract(
        &self,
        from_pkscript: String,
        contract_address: Option<AddressWrapper>,
        contract_inscription_id: Option<String>,
        data: BytesWrapper,
        timestamp: u64,
        hash: B256Wrapper,
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
        amount: U256Wrapper,
        timestamp: u64,
        hash: B256Wrapper,
        tx_idx: u64,
        inscription_id: Option<String>,
    ) -> RpcResult<TxReceiptED>;

    /// Withdraws brc20 tokens from the given address
    #[method(name = "brc20_withdraw")]
    async fn withdraw(
        &self,
        from_pkscript: String,
        ticker: String,
        amount: U256Wrapper,
        timestamp: u64,
        hash: B256Wrapper,
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
        genesis_hash: B256Wrapper,
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
    async fn get_inscription_id_by_tx_hash(
        &self,
        transaction: B256Wrapper,
    ) -> RpcResult<Option<String>>;

    /// Finalises the block with the given parameters
    #[method(name = "brc20_finaliseBlock")]
    async fn finalise_block(
        &self,
        timestamp: u64,
        hash: B256Wrapper,
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
        block: B256Wrapper,
        is_full: Option<bool>,
    ) -> RpcResult<BlockResponseED>;

    /// Returns the transaction count by address and block number
    #[method(name = "eth_getTransactionCount")]
    async fn get_transaction_count(
        &self,
        account: AddressWrapper,
        block: String,
    ) -> RpcResult<String>;

    /// Returns the transaction count by block number
    #[method(name = "eth_getBlockTransactionCountByNumber")]
    async fn get_block_transaction_count_by_number(&self, block: String) -> RpcResult<String>;

    /// Returns the transaction count by block hash
    #[method(name = "eth_getBlockTransactionCountByHash")]
    async fn get_block_transaction_count_by_hash(&self, block: B256Wrapper) -> RpcResult<String>;

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
    async fn get_storage_at(
        &self,
        contract: AddressWrapper,
        location: U256Wrapper,
    ) -> RpcResult<String>;

    /// Returns the bytecode of the contract at the given address
    #[method(name = "eth_getCode")]
    async fn get_code(&self, contract: AddressWrapper) -> RpcResult<BytecodeED>;

    /// Returns the transaction receipt for the given transaction hash
    #[method(name = "eth_getTransactionReceipt")]
    async fn get_transaction_receipt(
        &self,
        transaction: B256Wrapper,
    ) -> RpcResult<Option<TxReceiptED>>;

    /// Returns the trace for the given transaction hash
    #[method(name = "debug_traceTransaction")]
    async fn debug_trace_transaction(&self, transaction: B256Wrapper)
        -> RpcResult<Option<TraceED>>;

    /// Returns the transaction by hash
    #[method(name = "eth_getTransactionByHash")]
    async fn get_transaction_by_hash(&self, transaction: B256Wrapper) -> RpcResult<Option<TxED>>;

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
        hash: B256Wrapper,
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
    async fn get_balance(&self, _address: AddressWrapper, _block: String) -> RpcResult<String> {
        Ok("0x0".to_string())
    }

    /// Returns the uncle count of the block at the given block number (0 in BRC20)
    #[method(name = "eth_getUncleCountByBlockNumber")]
    async fn get_uncle_count_by_block_number(&self, _number: u64) -> RpcResult<String> {
        Ok("0x0".to_string())
    }

    /// Returns the uncle count of the block at the given block hash (0 in BRC20)
    #[method(name = "eth_getUncleCountByBlockHash")]
    async fn get_uncle_count_by_block_hash(&self, _hash: B256Wrapper) -> RpcResult<String> {
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
        _hash: B256Wrapper,
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

#[derive(Debug, serde::Deserialize)]
pub struct EthCall {
    pub from: Option<AddressWrapper>,
    pub to: Option<AddressWrapper>,
    pub data: Option<BytesWrapper>,
    pub input: Option<BytesWrapper>,
}

impl EthCall {
    pub fn data_or_input(&self) -> Option<&BytesWrapper> {
        if let Some(data) = &self.data {
            Some(data)
        } else if let Some(input) = &self.input {
            Some(input)
        } else {
            None
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct GetLogsFilter {
    #[serde(rename = "fromBlock")]
    pub from_block: Option<String>,
    #[serde(rename = "toBlock")]
    pub to_block: Option<String>,
    pub address: Option<AddressWrapper>,
    pub topics: Option<Vec<B256Wrapper>>,
}

#[derive(Debug)]
pub struct U256Wrapper(U256);

impl U256Wrapper {
    pub fn value(&self) -> U256 {
        self.0
    }
}

impl<'de> Deserialize<'de> for U256Wrapper {
    fn deserialize<D>(deserializer: D) -> Result<U256Wrapper, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.starts_with("0x") {
            let u256 = U256::from_str_radix(&s[2..], 16).map_err(serde::de::Error::custom)?;
            Ok(U256Wrapper(u256))
        } else {
            let u256 = U256::from_str_radix(&s, 10).map_err(serde::de::Error::custom)?;
            Ok(U256Wrapper(u256))
        }
    }
}

#[derive(Debug)]
pub struct B256Wrapper(B256);

impl B256Wrapper {
    #[cfg(test)]
    pub fn new(b256: B256) -> Self {
        Self(b256)
    }

    pub fn value(&self) -> B256 {
        self.0
    }
}

impl<'de> Deserialize<'de> for B256Wrapper {
    fn deserialize<D>(deserializer: D) -> Result<B256Wrapper, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let b256 = FixedBytes::from_hex(&s).map_err(serde::de::Error::custom)?;
        Ok(B256Wrapper(b256))
    }
}

#[derive(Debug)]
pub struct AddressWrapper(Address);

impl AddressWrapper {
    pub fn value(&self) -> Address {
        self.0
    }
}

impl<'de> Deserialize<'de> for AddressWrapper {
    fn deserialize<D>(deserializer: D) -> Result<AddressWrapper, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let Ok(s) = String::deserialize(deserializer) else {
            return Ok(AddressWrapper(*INVALID_ADDRESS));
        };
        match Address::from_str(&s) {
            Ok(address) => Ok(AddressWrapper(address)),
            Err(_) => Ok(AddressWrapper(*INVALID_ADDRESS)),
        }
    }
}

#[derive(Debug)]
pub struct BytesWrapper(Option<Bytes>);

impl BytesWrapper {
    pub fn new(bytes: Bytes) -> Self {
        Self(Some(bytes))
    }

    pub fn empty() -> Self {
        Self(None)
    }

    pub fn value(&self) -> Option<&Bytes> {
        self.0.as_ref()
    }
}

impl<'de> Deserialize<'de> for BytesWrapper {
    fn deserialize<D>(deserializer: D) -> Result<BytesWrapper, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let Ok(s) = String::deserialize(deserializer) else {
            return Ok(BytesWrapper::empty());
        };
        match Bytes::from_hex(&s) {
            Ok(bytes) => Ok(BytesWrapper::new(bytes)),
            Err(_) => {
                return Ok(BytesWrapper::empty());
            }
        }
    }
}
