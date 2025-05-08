use std::error::Error;
use std::net::SocketAddr;

use alloy_primitives::Bytes;
use hyper::Method;
use jsonrpsee::core::middleware::RpcServiceBuilder;
use jsonrpsee::core::{async_trait, RpcResult};
use jsonrpsee::server::{Server, ServerHandle};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::validate_request::ValidateRequestHeaderLayer;
use tracing::{info, instrument, warn};

use crate::brc20_controller::{
    decode_brc20_balance_result, load_brc20_balance_tx, load_brc20_burn_tx, load_brc20_mint_tx,
};
use crate::db::types::{
    AddressED, BlockResponseED, BytecodeED, LogED, TraceED, TxED, TxReceiptED, B256ED, U256ED,
};
use crate::evm::utils::get_evm_address;
use crate::server::api::{
    Brc20ProgApiServer, EncodedBytes, EthCall, GetLogsFilter, INDEXER_METHODS, INVALID_ADDRESS,
};
use crate::server::auth::{HttpNonBlockingAuth, RpcAuthMiddleware};
use crate::server::engine::BRC20ProgEngine;
use crate::server::error::{
    wrap_hex_error, wrap_rpc_error, wrap_rpc_error_string, wrap_rpc_error_string_with_data,
};
use crate::server::types::TxInfo;
use crate::Brc20ProgConfig;

pub struct RpcServer {
    engine: BRC20ProgEngine,
}

impl RpcServer {
    fn parse_block_number(&self, number: &str) -> Result<u64, Box<dyn Error>> {
        if number == "latest" || number == "safe" || number == "finalized" {
            self.engine.get_latest_block_height()
        } else if number == "pending" {
            self.engine.get_next_block_height()
        } else if number == "earliest" {
            Ok(0)
        } else if number.starts_with("0x") {
            u64::from_str_radix(&number[2..], 16).map_err(|_| "Invalid block number".into())
        } else {
            number.parse().map_err(|_| "Invalid block number".into())
        }
    }
}

fn log_call() {
    info!("rpc.request");
}

#[async_trait]
impl Brc20ProgApiServer for RpcServer {
    #[instrument(name = "brc20_mine", skip(self), level = "error")]
    async fn mine(&self, block_count: u64, timestamp: u64) -> RpcResult<()> {
        log_call();
        self.engine
            .mine_blocks(block_count, timestamp)
            .map_err(wrap_rpc_error)
    }

    #[instrument(name = "brc20_deposit", skip(self), level = "error")]
    async fn deposit(
        &self,
        to_pkscript: String,
        ticker: String,
        amount: U256ED,
        timestamp: u64,
        hash: B256ED,
        tx_idx: u64,
        inscription_id: Option<String>,
    ) -> RpcResult<TxReceiptED> {
        log_call();

        let to_pkscript = hex::decode(to_pkscript).map_err(wrap_hex_error)?.into();

        self.engine
            .add_tx_to_block(
                timestamp,
                &load_brc20_mint_tx(
                    ticker_as_bytes(&ticker),
                    get_evm_address(&to_pkscript),
                    amount.uint,
                ),
                tx_idx,
                self.engine
                    .get_next_block_height()
                    .map_err(wrap_rpc_error)?,
                hash.bytes,
                inscription_id,
                Some(u64::MAX),
            )
            .map_err(wrap_rpc_error)
    }

    #[instrument(name = "brc20_withdraw", skip(self), level = "error")]
    async fn withdraw(
        &self,
        from_pkscript: String,
        ticker: String,
        amount: U256ED,
        timestamp: u64,
        hash: B256ED,
        tx_idx: u64,
        inscription_id: Option<String>,
    ) -> RpcResult<TxReceiptED> {
        log_call();

        let from_pkscript = hex::decode(from_pkscript).map_err(wrap_hex_error)?.into();

        self.engine
            .add_tx_to_block(
                timestamp,
                &load_brc20_burn_tx(
                    ticker_as_bytes(&ticker),
                    get_evm_address(&from_pkscript),
                    amount.uint,
                ),
                tx_idx,
                self.engine
                    .get_next_block_height()
                    .map_err(wrap_rpc_error)?,
                hash.bytes,
                inscription_id,
                Some(u64::MAX),
            )
            .map_err(wrap_rpc_error)
    }

    #[instrument(name = "brc20_balance", skip(self), level = "error")]
    async fn balance(&self, pkscript: String, ticker: String) -> RpcResult<String> {
        log_call();

        let pkscript = hex::decode(pkscript).map_err(wrap_hex_error)?.into();

        self.engine
            .read_contract(&load_brc20_balance_tx(
                ticker_as_bytes(&ticker),
                get_evm_address(&pkscript),
            ))
            .map(|receipt| {
                format!(
                    "0x{:x}",
                    decode_brc20_balance_result(receipt.result_bytes.map(|x| x.bytes).as_ref())
                )
            })
            .map_err(wrap_rpc_error)
    }

    #[instrument(name = "brc20_initialise", skip(self), level = "error")]
    async fn initialise(
        &self,
        genesis_hash: B256ED,
        genesis_timestamp: u64,
        genesis_height: u64,
    ) -> RpcResult<()> {
        log_call();
        self.engine
            .initialise(genesis_hash.bytes, genesis_timestamp, genesis_height)
            .map_err(wrap_rpc_error)
    }

    #[instrument(
        name = "brc20_getTxReceiptByInscriptionId",
        skip(self),
        level = "error"
    )]
    async fn get_transaction_receipt_by_inscription_id(
        &self,
        inscription_id: String,
    ) -> RpcResult<Option<TxReceiptED>> {
        log_call();
        self.engine
            .get_transaction_receipt_by_inscription_id(inscription_id)
            .map_err(wrap_rpc_error)
    }

    #[instrument(
        name = "brc20_getInscriptionIdByContractAddress",
        skip(self),
        level = "error"
    )]
    async fn get_inscription_id_by_contract_address(
        &self,
        contract_address: AddressED,
    ) -> RpcResult<Option<String>> {
        log_call();
        self.engine
            .get_inscription_id_by_contract_address(contract_address.address)
            .map_err(wrap_rpc_error)
    }

    #[instrument(name = "brc20_getInscriptionIdByTxHash", skip(self), level = "error")]
    async fn get_inscription_id_by_tx_hash(
        &self,
        transaction: B256ED,
    ) -> RpcResult<Option<String>> {
        log_call();
        self.engine
            .get_transaction_by_hash(transaction.bytes)
            .map(|tx| tx.and_then(|tx| tx.inscription_id))
            .map_err(wrap_rpc_error)
    }

    #[instrument(name = "brc20_deploy", skip(self, data), level = "error")]
    async fn deploy_contract(
        &self,
        from_pkscript: String,
        data: EncodedBytes,
        timestamp: u64,
        hash: B256ED,
        tx_idx: u64,
        inscription_id: Option<String>,
        inscription_byte_len: Option<u64>,
    ) -> RpcResult<TxReceiptED> {
        log_call();

        let block_height = self
            .engine
            .get_next_block_height()
            .map_err(wrap_rpc_error)?;

        let from_pkscript = hex::decode(from_pkscript).map_err(wrap_hex_error)?.into();

        let data = data.value_inscription(block_height);
        let to = if data.is_some() {
            None
        } else {
            Some(*INVALID_ADDRESS) // If data is not valid, send transaction to 0xdead
        };

        self.engine
            .add_tx_to_block(
                timestamp,
                &TxInfo {
                    from: get_evm_address(&from_pkscript),
                    to,
                    data: data.unwrap_or_default().clone(),
                },
                tx_idx,
                block_height,
                hash.bytes,
                inscription_id,
                inscription_byte_len,
            )
            .map_err(wrap_rpc_error)
    }

    #[instrument(name = "brc20_call", skip(self, data), level = "error")]
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
    ) -> RpcResult<Option<TxReceiptED>> {
        log_call();

        let from_pkscript = hex::decode(from_pkscript).map_err(wrap_hex_error)?.into();

        let block_height = self
            .engine
            .get_next_block_height()
            .map_err(wrap_rpc_error)?;

        let data = data.value_inscription(block_height);

        let derived_contract_address = if !data.is_some() {
            *INVALID_ADDRESS
        } else if let Some(contract_inscription_id) = contract_inscription_id {
            self.engine
                .get_contract_address_by_inscription_id(contract_inscription_id)
                .unwrap_or(None)
                .unwrap_or(*INVALID_ADDRESS)
        } else {
            contract_address
                .map(|x| x.address)
                .unwrap_or(*INVALID_ADDRESS)
        };

        self.engine
            .add_tx_to_block(
                timestamp,
                &TxInfo {
                    from: get_evm_address(&from_pkscript),
                    to: derived_contract_address.into(),
                    data: data.unwrap_or_default().clone(),
                },
                tx_idx,
                block_height,
                hash.bytes,
                inscription_id,
                inscription_byte_len,
            )
            .map(|receipt| Some(receipt))
            .map_err(wrap_rpc_error)
    }

    #[instrument(name = "brc20_finaliseBlock", skip(self), level = "error")]
    async fn finalise_block(
        &self,
        timestamp: u64,
        hash: B256ED,
        block_tx_count: u64,
    ) -> RpcResult<()> {
        log_call();
        let block_height = self
            .engine
            .get_next_block_height()
            .map_err(wrap_rpc_error)?;
        self.engine
            .finalise_block(timestamp, block_height, hash.bytes, block_tx_count)
            .map_err(wrap_rpc_error)
    }

    #[instrument(name = "brc20_reorg", skip(self), level = "error")]
    async fn reorg(&self, latest_valid_block_number: u64) -> RpcResult<()> {
        warn!("Reorg!");
        self.engine
            .reorg(latest_valid_block_number)
            .map_err(wrap_rpc_error)
    }

    #[instrument(name = "brc20_commitToDatabase", skip(self), level = "error")]
    async fn commit_to_database(&self) -> RpcResult<()> {
        log_call();
        self.engine.commit_to_db().map_err(wrap_rpc_error)
    }

    #[instrument(name = "brc20_clearCaches", skip(self), level = "error")]
    async fn clear_caches(&self) -> RpcResult<()> {
        log_call();
        self.engine.clear_caches().map_err(wrap_rpc_error)
    }

    #[instrument(name = "eth_blockNumber", skip(self), level = "error")]
    async fn block_number(&self) -> RpcResult<String> {
        log_call();
        Ok(format!(
            "0x{:x}",
            self.engine
                .get_latest_block_height()
                .map_err(wrap_rpc_error)?
        ))
    }

    #[instrument(name = "eth_getBlockByNumber", skip(self), level = "error")]
    async fn get_block_by_number(
        &self,
        block: String,
        is_full: Option<bool>,
    ) -> RpcResult<BlockResponseED> {
        log_call();
        let block_number = self.parse_block_number(&block).map_err(wrap_rpc_error)?;
        if let Some(block) = self
            .engine
            .get_block_by_number(block_number, is_full.unwrap_or(false))
            .map_err(wrap_rpc_error)?
        {
            Ok(block)
        } else {
            Err(wrap_rpc_error_string("Block not found"))
        }
    }

    #[instrument(name = "eth_getBlockByHash", skip(self), level = "error")]
    async fn get_block_by_hash(
        &self,
        block: B256ED,
        is_full: Option<bool>,
    ) -> RpcResult<BlockResponseED> {
        log_call();
        if let Some(block) = self
            .engine
            .get_block_by_hash(block.bytes, is_full.unwrap_or(false))
            .map_err(wrap_rpc_error)?
        {
            Ok(block)
        } else {
            Err(wrap_rpc_error_string("Block not found"))
        }
    }

    #[instrument(name = "eth_getTransactionCount", skip(self), level = "error")]
    async fn get_transaction_count(&self, account: AddressED, block: String) -> RpcResult<String> {
        log_call();
        let block_number = self.parse_block_number(&block).map_err(wrap_rpc_error)?;
        self.engine
            .get_transaction_count(account.address, block_number)
            .map(|count| format!("0x{:x}", count))
            .map_err(wrap_rpc_error)
    }

    #[instrument(
        name = "eth_getBlockTransactionCountByNumber",
        skip(self),
        level = "error"
    )]
    async fn get_block_transaction_count_by_number(&self, block: String) -> RpcResult<String> {
        log_call();
        let block_number = self.parse_block_number(&block).map_err(wrap_rpc_error)?;
        self.engine
            .get_block_transaction_count_by_number(block_number)
            .map(|count| format!("0x{:x}", count))
            .map_err(wrap_rpc_error)
    }

    #[instrument(
        name = "eth_getBlockTransactionCountByHash",
        skip(self),
        level = "error"
    )]
    async fn get_block_transaction_count_by_hash(&self, block: B256ED) -> RpcResult<String> {
        log_call();
        self.engine
            .get_block_transaction_count_by_hash(block.bytes)
            .map(|count| format!("0x{:x}", count))
            .map_err(wrap_rpc_error)
    }

    #[instrument(name = "eth_getLogs", skip(self), level = "error")]
    async fn get_logs(&self, filter: GetLogsFilter) -> RpcResult<Vec<LogED>> {
        log_call();
        let from_block = filter
            .from_block
            .clone()
            .and_then(|from| self.parse_block_number(&from).ok());
        let to_block = filter
            .to_block
            .clone()
            .and_then(|to| self.parse_block_number(&to).ok());
        Ok(self
            .engine
            .get_logs(
                from_block,
                to_block,
                filter.address.clone().map(|x| x.address),
                filter.topics_as_b256(),
            )
            .map_err(wrap_rpc_error)?)
    }

    #[instrument(name = "eth_call", skip(self), level = "error")]
    async fn call(&self, call: EthCall, _: Option<String>) -> RpcResult<String> {
        log_call();
        let Some(data) = call.data_or_input() else {
            return Err(wrap_rpc_error_string("No data or input provided"));
        };
        let receipt = self.engine.read_contract(&TxInfo {
            from: call
                .from
                .as_ref()
                .map(|x| x.address)
                .unwrap_or(*INVALID_ADDRESS),
            to: call.to.as_ref().map(|x| x.address),
            data: data.value_eth().unwrap_or_default().clone(),
        });
        let Ok(receipt) = receipt else {
            return Err(wrap_rpc_error_string("Call failed"));
        };
        let data_string = receipt
            .result_bytes
            .map(|x| x.bytes)
            .unwrap_or(Bytes::new())
            .to_string();
        if receipt.status.uint.is_zero() {
            return Err(wrap_rpc_error_string_with_data("Call failed", data_string));
        }
        Ok(data_string)
    }

    #[instrument(name = "eth_estimateGas", skip(self), level = "error")]
    async fn estimate_gas(&self, call: EthCall, _: Option<String>) -> RpcResult<String> {
        log_call();
        let Some(data) = call.data_or_input() else {
            return Err(wrap_rpc_error_string("No data or input provided"));
        };
        let receipt = self.engine.read_contract(&TxInfo {
            from: call
                .from
                .as_ref()
                .map(|x| x.address)
                .unwrap_or(*INVALID_ADDRESS),
            to: call.to.as_ref().map(|x| x.address),
            data: data.value_eth().unwrap_or_default().clone(),
        });
        let Ok(receipt) = receipt else {
            return Err(wrap_rpc_error_string("Call failed"));
        };
        let data_string = receipt
            .result_bytes
            .map(|x| x.bytes)
            .unwrap_or(Bytes::new())
            .to_string();
        if receipt.status.uint.is_zero() {
            return Err(wrap_rpc_error_string_with_data("Call failed", data_string));
        }
        let gas_used: u64 = receipt.gas_used.into();
        Ok(format!("0x{:x}", gas_used))
    }

    #[instrument(name = "eth_getStorageAt", skip(self), level = "error")]
    async fn get_storage_at(&self, contract: AddressED, location: U256ED) -> RpcResult<String> {
        log_call();
        Ok(format!(
            "0x{:x}",
            self.engine
                .get_storage_at(contract.address, location.uint)
                .map_err(wrap_rpc_error)?
        ))
    }

    #[instrument(name = "eth_getCode", skip(self), level = "error")]
    async fn get_code(&self, contract: AddressED) -> RpcResult<BytecodeED> {
        log_call();
        if let Some(bytecode) = self
            .engine
            .get_contract_bytecode(contract.address)
            .map_err(wrap_rpc_error)?
        {
            Ok(bytecode)
        } else {
            Err(wrap_rpc_error_string("Contract bytecode not found"))
        }
    }

    #[instrument(name = "eth_getTransactionReceipt", skip(self), level = "error")]
    async fn get_transaction_receipt(&self, transaction: B256ED) -> RpcResult<Option<TxReceiptED>> {
        log_call();
        self.engine
            .get_transaction_receipt(transaction.bytes)
            .map_err(wrap_rpc_error)
    }

    #[instrument(name = "debug_traceTransaction", skip(self), level = "error")]
    async fn debug_trace_transaction(&self, transaction: B256ED) -> RpcResult<Option<TraceED>> {
        log_call();
        self.engine
            .get_trace(transaction.bytes)
            .map_err(wrap_rpc_error)
    }

    #[instrument(name = "eth_getTransactionByHash", skip(self), level = "error")]
    async fn get_transaction_by_hash(&self, transaction: B256ED) -> RpcResult<Option<TxED>> {
        log_call();
        self.engine
            .get_transaction_by_hash(transaction.bytes)
            .map_err(wrap_rpc_error)
    }

    #[instrument(
        name = "eth_getTransactionByBlockNumberAndIndex",
        skip(self),
        level = "error"
    )]
    async fn get_transaction_by_block_number_and_index(
        &self,
        block_number: u64,
        tx_idx: Option<u64>,
    ) -> RpcResult<Option<TxED>> {
        log_call();
        self.engine
            .get_transaction_by_block_number_and_index(block_number, tx_idx.unwrap_or(0))
            .map_err(wrap_rpc_error)
    }

    #[instrument(
        name = "eth_getTransactionByBlockHashAndIndex",
        skip(self),
        level = "error"
    )]
    async fn get_transaction_by_block_hash_and_index(
        &self,
        block_hash: B256ED,
        tx_idx: Option<u64>,
    ) -> RpcResult<Option<TxED>> {
        log_call();
        self.engine
            .get_transaction_by_block_hash_and_index(block_hash.bytes, tx_idx.unwrap_or(0))
            .map_err(wrap_rpc_error)
    }
}

pub async fn start_rpc_server(
    engine: BRC20ProgEngine,
    config: Brc20ProgConfig,
) -> Result<ServerHandle, Box<dyn Error>> {
    let cors = CorsLayer::new()
        // Allow `POST` when accessing the resource
        .allow_methods([Method::POST])
        // Allow requests from any origin
        .allow_origin(Any)
        .allow_headers([hyper::header::CONTENT_TYPE]);

    let http_middleware =
        ServiceBuilder::new()
            .layer(cors)
            .layer(ValidateRequestHeaderLayer::custom(
                if !config.brc20_prog_rpc_server_enable_auth {
                    HttpNonBlockingAuth::allow()
                } else {
                    let Some(rpc_username) = config.brc20_prog_rpc_server_user else {
                        return Err(
                "RPC username environment variable is required when authentication is enabled"
                    .into(),
            );
                    };
                    let Some(rpc_password) = config.brc20_prog_rpc_server_password else {
                        return Err(
                "RPC password environment variable is required when authentication is enabled"
                    .into(),
            );
                    };
                    HttpNonBlockingAuth::new(&rpc_username, &rpc_password)
                },
            ));
    let rpc_middleware = RpcServiceBuilder::new()
        .rpc_logger(1024)
        .layer_fn(|service| RpcAuthMiddleware::new(service, &*INDEXER_METHODS));
    let module = RpcServer { engine }.into_rpc();

    let handle = Server::builder()
        .set_http_middleware(http_middleware)
        .set_rpc_middleware(rpc_middleware)
        .build(config.brc20_prog_rpc_server_url.parse::<SocketAddr>()?)
        .await?
        .start(module);

    Ok(handle)
}

fn ticker_as_bytes(ticker: &str) -> Bytes {
    let ticker_lowercase = ticker.to_lowercase();
    Bytes::from(ticker_lowercase.as_bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use alloy_primitives::B256;
    use tempfile::TempDir;

    use super::*;
    use crate::db::DB;
    use crate::server::engine::BRC20ProgEngine;

    fn create_test_server() -> RpcServer {
        let temp_dir = TempDir::new().unwrap();
        let db = DB::new(temp_dir.path()).unwrap();
        RpcServer {
            engine: BRC20ProgEngine::new(db),
        }
    }

    #[test]
    fn test_ticker_as_bytes() {
        assert_eq!(
            ticker_as_bytes("BRC20"),
            Bytes::from(vec![0x62, 0x72, 0x63, 0x32, 0x30])
        );
        assert_eq!(
            ticker_as_bytes("brc20"),
            Bytes::from(vec![0x62, 0x72, 0x63, 0x32, 0x30])
        );
    }

    #[tokio::test]
    async fn test_parse_block_number() {
        let server = create_test_server();

        let _ = server.initialise([1; 32].into(), 20, 0).await;

        assert_eq!(server.parse_block_number("latest").unwrap(), 0);
        assert_eq!(server.parse_block_number("safe").unwrap(), 0);
        assert_eq!(server.parse_block_number("finalized").unwrap(), 0);
        assert_eq!(server.parse_block_number("pending").unwrap(), 1);
        assert_eq!(server.parse_block_number("earliest").unwrap(), 0);
        assert_eq!(server.parse_block_number("0x1").unwrap(), 1);
        assert_eq!(server.parse_block_number("1").unwrap(), 1);
    }

    #[test]
    fn test_parse_block_number_invalid() {
        let server = create_test_server();
        assert!(server.parse_block_number("invalid").is_err());
        assert!(server.parse_block_number("0xinvalid").is_err());
    }

    #[tokio::test]
    async fn test_initialise() {
        let server = create_test_server();
        let _ = server.initialise([1; 32].into(), 20, 0).await;

        assert_eq!(server.engine.get_latest_block_height().unwrap(), 0);
        assert_eq!(server.engine.get_next_block_height().unwrap(), 1);
        assert_eq!(
            server
                .engine
                .get_block_by_number(0, false)
                .unwrap()
                .unwrap()
                .number,
            0u64.into()
        );
        assert_eq!(server.engine.get_block_by_number(1, true).unwrap(), None);
        assert_eq!(
            server
                .engine
                .get_block_by_hash(B256::from_slice(&[1; 32]), true)
                .unwrap()
                .unwrap()
                .number,
            0u64.into()
        )
    }

    #[tokio::test]
    async fn test_mine() {
        let server = create_test_server();
        let _ = server.initialise([1; 32].into(), 20, 0).await;

        assert_eq!(server.engine.get_latest_block_height().unwrap(), 0);
        assert_eq!(server.engine.get_next_block_height().unwrap(), 1);

        assert!(server.mine(1, 20).await.is_ok());
        assert_eq!(server.engine.get_latest_block_height().unwrap(), 1);
        assert_eq!(server.engine.get_next_block_height().unwrap(), 2);
    }

    #[tokio::test]
    async fn test_invalid_tx() {
        let server = create_test_server();

        let result = server
            .call_contract(
                "deadbeef".to_string(),
                None,
                None,
                EncodedBytes::empty(),
                20,
                [1; 32].into(),
                0,
                None,
                Some(1000),
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().unwrap().to.unwrap().address,
            *INVALID_ADDRESS
        );
    }
}
