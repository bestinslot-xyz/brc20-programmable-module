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
use tracing::{event, instrument, Level};

use crate::brc20_controller::{
    decode_brc20_balance_result, load_brc20_balance_tx, load_brc20_burn_tx, load_brc20_mint_tx,
};
use crate::db::types::{BlockResponseED, BytecodeED, LogED, TraceED, TxED, TxReceiptED};
use crate::evm::utils::get_evm_address;
use crate::server::api::{
    AddressWrapper, B256Wrapper, Brc20ProgApiServer, BytesWrapper, EthCall, GetLogsFilter,
    U256Wrapper, INDEXER_METHODS, INVALID_ADDRESS,
};
use crate::server::auth::{HttpNonBlockingAuth, RpcAuthMiddleware};
use crate::server::engine::BRC20ProgEngine;
use crate::server::error::{
    wrap_hex_error, wrap_rpc_error, wrap_rpc_error_string, wrap_rpc_error_string_with_data,
};
use crate::server::types::TxInfo;

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

#[async_trait]
impl Brc20ProgApiServer for RpcServer {
    #[instrument(skip(self))]
    async fn mine(&self, block_count: u64, timestamp: u64) -> RpcResult<()> {
        event!(Level::INFO, "Mining empty blocks");
        self.engine
            .mine_blocks(block_count, timestamp)
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self))]
    async fn deposit(
        &self,
        to_pkscript: String,
        ticker: String,
        amount: U256Wrapper,
        timestamp: u64,
        hash: B256Wrapper,
        tx_idx: u64,
        inscription_id: Option<String>,
    ) -> RpcResult<TxReceiptED> {
        event!(Level::INFO, "Depositing");

        let to_pkscript = hex::decode(to_pkscript).map_err(wrap_hex_error)?.into();

        self.engine
            .add_tx_to_block(
                timestamp,
                &load_brc20_mint_tx(
                    ticker_as_bytes(&ticker),
                    get_evm_address(&to_pkscript),
                    amount.value(),
                ),
                tx_idx,
                self.engine
                    .get_next_block_height()
                    .map_err(wrap_rpc_error)?,
                hash.value(),
                inscription_id,
                Some(u64::MAX),
            )
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self))]
    async fn withdraw(
        &self,
        from_pkscript: String,
        ticker: String,
        amount: U256Wrapper,
        timestamp: u64,
        hash: B256Wrapper,
        tx_idx: u64,
        inscription_id: Option<String>,
    ) -> RpcResult<TxReceiptED> {
        event!(Level::INFO, "Withdrawing");

        let from_pkscript = hex::decode(from_pkscript).map_err(wrap_hex_error)?.into();

        self.engine
            .add_tx_to_block(
                timestamp,
                &load_brc20_burn_tx(
                    ticker_as_bytes(&ticker),
                    get_evm_address(&from_pkscript),
                    amount.value(),
                ),
                tx_idx,
                self.engine
                    .get_next_block_height()
                    .map_err(wrap_rpc_error)?,
                hash.value(),
                inscription_id,
                Some(u64::MAX),
            )
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self))]
    async fn balance(&self, pkscript: String, ticker: String) -> RpcResult<String> {
        event!(Level::INFO, "Checking balance");

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

    #[instrument(skip(self))]
    async fn initialise(
        &self,
        genesis_hash: B256Wrapper,
        genesis_timestamp: u64,
        genesis_height: u64,
    ) -> RpcResult<()> {
        event!(Level::INFO, "Initialising server");
        self.engine
            .initialise(genesis_hash.value(), genesis_timestamp, genesis_height)
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self))]
    async fn get_transaction_receipt_by_inscription_id(
        &self,
        inscription_id: String,
    ) -> RpcResult<Option<TxReceiptED>> {
        event!(Level::INFO, "Getting transaction receipt by inscription id");
        self.engine
            .get_transaction_receipt_by_inscription_id(inscription_id)
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self))]
    async fn get_inscription_id_by_tx_hash(
        &self,
        transaction: B256Wrapper,
    ) -> RpcResult<Option<String>> {
        event!(Level::INFO, "Getting inscription id by transaction hash");
        self.engine
            .get_transaction_by_hash(transaction.value())
            .map(|tx| tx.and_then(|tx| tx.inscription_id))
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self, data))]
    async fn deploy_contract(
        &self,
        from_pkscript: String,
        data: BytesWrapper,
        timestamp: u64,
        hash: B256Wrapper,
        tx_idx: u64,
        inscription_id: Option<String>,
        inscription_byte_len: Option<u64>,
    ) -> RpcResult<TxReceiptED> {
        event!(Level::INFO, "Deploying contract");

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
                hash.value(),
                inscription_id,
                inscription_byte_len,
            )
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self, data))]
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
    ) -> RpcResult<Option<TxReceiptED>> {
        event!(Level::INFO, "Calling contract");

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
                .map(|x| x.value())
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
                hash.value(),
                inscription_id,
                inscription_byte_len,
            )
            .map(|receipt| Some(receipt))
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self))]
    async fn finalise_block(
        &self,
        timestamp: u64,
        hash: B256Wrapper,
        block_tx_count: u64,
    ) -> RpcResult<()> {
        let block_height = self
            .engine
            .get_next_block_height()
            .map_err(wrap_rpc_error)?;
        event!(Level::INFO, "Finalising block {}", block_height);
        self.engine
            .finalise_block(timestamp, block_height, hash.value(), block_tx_count)
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self))]
    async fn reorg(&self, latest_valid_block_number: u64) -> RpcResult<()> {
        event!(Level::WARN, "Reorg!");
        self.engine
            .reorg(latest_valid_block_number)
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self))]
    async fn commit_to_database(&self) -> RpcResult<()> {
        event!(Level::INFO, "Committing to database");
        self.engine.commit_to_db().map_err(wrap_rpc_error)
    }

    #[instrument(skip(self))]
    async fn clear_caches(&self) -> RpcResult<()> {
        event!(Level::INFO, "Clearing caches");
        self.engine.clear_caches().map_err(wrap_rpc_error)
    }

    async fn block_number(&self) -> RpcResult<String> {
        Ok(format!(
            "0x{:x}",
            self.engine
                .get_latest_block_height()
                .map_err(wrap_rpc_error)?
        ))
    }

    #[instrument(skip(self))]
    async fn get_block_by_number(
        &self,
        block: String,
        is_full: Option<bool>,
    ) -> RpcResult<BlockResponseED> {
        let block_number = self.parse_block_number(&block).map_err(wrap_rpc_error)?;
        event!(Level::INFO, "Getting block by number: {}", block_number);
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

    #[instrument(skip(self))]
    async fn get_block_by_hash(
        &self,
        block: B256Wrapper,
        is_full: Option<bool>,
    ) -> RpcResult<BlockResponseED> {
        event!(Level::INFO, "Getting block by number");
        if let Some(block) = self
            .engine
            .get_block_by_hash(block.value(), is_full.unwrap_or(false))
            .map_err(wrap_rpc_error)?
        {
            Ok(block)
        } else {
            Err(wrap_rpc_error_string("Block not found"))
        }
    }

    #[instrument(skip(self))]
    async fn get_transaction_count(
        &self,
        account: AddressWrapper,
        block: String,
    ) -> RpcResult<String> {
        event!(Level::INFO, "Getting transaction count");
        let block_number = self.parse_block_number(&block).map_err(wrap_rpc_error)?;
        self.engine
            .get_transaction_count(account.value(), block_number)
            .map(|count| format!("0x{:x}", count))
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self))]
    async fn get_block_transaction_count_by_number(&self, block: String) -> RpcResult<String> {
        event!(Level::INFO, "Getting block transaction count");
        let block_number = self.parse_block_number(&block).map_err(wrap_rpc_error)?;
        self.engine
            .get_block_transaction_count_by_number(block_number)
            .map(|count| format!("0x{:x}", count))
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self))]
    async fn get_block_transaction_count_by_hash(&self, block: B256Wrapper) -> RpcResult<String> {
        event!(Level::INFO, "Getting block transaction count");
        self.engine
            .get_block_transaction_count_by_hash(block.value())
            .map(|count| format!("0x{:x}", count))
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self))]
    async fn get_logs(&self, filter: GetLogsFilter) -> RpcResult<Vec<LogED>> {
        event!(Level::INFO, "Getting logs");
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
                filter.address.clone().map(|x| x.value()),
                filter.topics_as_b256(),
            )
            .map_err(wrap_rpc_error)?)
    }

    #[instrument(skip(self))]
    async fn call(&self, call: EthCall, _: Option<String>) -> RpcResult<String> {
        event!(Level::INFO, "Calling contract");
        let Some(data) = call.data_or_input() else {
            return Err(wrap_rpc_error_string("No data or input provided"));
        };
        let receipt = self.engine.read_contract(&TxInfo {
            from: call
                .from
                .as_ref()
                .map(|x| x.value())
                .unwrap_or(*INVALID_ADDRESS),
            to: call.to.as_ref().map(|x| x.value()),
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
        if receipt.status == 0 {
            return Err(wrap_rpc_error_string_with_data("Call failed", data_string));
        }
        Ok(data_string)
    }

    #[instrument(skip(self))]
    async fn estimate_gas(&self, call: EthCall, _: Option<String>) -> RpcResult<String> {
        event!(Level::INFO, "Estimating gas");
        let Some(data) = call.data_or_input() else {
            return Err(wrap_rpc_error_string("No data or input provided"));
        };
        let receipt = self.engine.read_contract(&TxInfo {
            from: call
                .from
                .as_ref()
                .map(|x| x.value())
                .unwrap_or(*INVALID_ADDRESS),
            to: call.to.as_ref().map(|x| x.value()),
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
        if receipt.status == 0 {
            return Err(wrap_rpc_error_string_with_data("Call failed", data_string));
        }
        let gas_used: u64 = receipt.gas_used.into();
        Ok(format!("0x{:x}", gas_used))
    }

    #[instrument(skip(self))]
    async fn get_storage_at(
        &self,
        contract: AddressWrapper,
        location: U256Wrapper,
    ) -> RpcResult<String> {
        event!(Level::INFO, "Getting storage value");
        Ok(format!(
            "0x{:x}",
            self.engine
                .get_storage_at(contract.value(), location.value())
                .map_err(wrap_rpc_error)?
        ))
    }

    #[instrument(skip(self))]
    async fn get_code(&self, contract: AddressWrapper) -> RpcResult<BytecodeED> {
        event!(Level::INFO, "Getting contract code");
        if let Some(bytecode) = self
            .engine
            .get_contract_bytecode(contract.value())
            .map_err(wrap_rpc_error)?
        {
            Ok(bytecode)
        } else {
            Err(wrap_rpc_error_string("Contract bytecode not found"))
        }
    }

    #[instrument(skip(self))]
    async fn get_transaction_receipt(
        &self,
        transaction: B256Wrapper,
    ) -> RpcResult<Option<TxReceiptED>> {
        event!(Level::INFO, "Getting transaction receipt");
        self.engine
            .get_transaction_receipt(transaction.value())
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self))]
    async fn debug_trace_transaction(
        &self,
        transaction: B256Wrapper,
    ) -> RpcResult<Option<TraceED>> {
        event!(Level::INFO, "Retrieving transaction trace");
        self.engine
            .get_trace(transaction.value())
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self))]
    async fn get_transaction_by_hash(&self, transaction: B256Wrapper) -> RpcResult<Option<TxED>> {
        event!(Level::INFO, "Getting transaction by hash");
        self.engine
            .get_transaction_by_hash(transaction.value())
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self))]
    async fn get_transaction_by_block_number_and_index(
        &self,
        block_number: u64,
        tx_idx: Option<u64>,
    ) -> RpcResult<Option<TxED>> {
        event!(Level::INFO, "Getting transaction by block number and index");
        self.engine
            .get_transaction_by_block_number_and_index(block_number, tx_idx.unwrap_or(0))
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self))]
    async fn get_transaction_by_block_hash_and_index(
        &self,
        block_hash: B256Wrapper,
        tx_idx: Option<u64>,
    ) -> RpcResult<Option<TxED>> {
        event!(Level::INFO, "Getting transaction by block hash and index");
        self.engine
            .get_transaction_by_block_hash_and_index(block_hash.value(), tx_idx.unwrap_or(0))
            .map_err(wrap_rpc_error)
    }
}

pub async fn start_rpc_server(
    engine: BRC20ProgEngine,
    addr: String,
    use_auth: bool,
    rpc_username: Option<&String>,
    rpc_password: Option<&String>,
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
            .layer(ValidateRequestHeaderLayer::custom(if !use_auth {
                HttpNonBlockingAuth::allow()
            } else {
                let Some(rpc_username) = rpc_username else {
                    return Err(
                "RPC username environment variable is required when authentication is enabled"
                    .into(),
            );
                };
                let Some(rpc_password) = rpc_password else {
                    return Err(
                "RPC password environment variable is required when authentication is enabled"
                    .into(),
            );
                };
                HttpNonBlockingAuth::new(rpc_username, rpc_password)
            }));
    let rpc_middleware = RpcServiceBuilder::new()
        .rpc_logger(1024)
        .layer_fn(|service| RpcAuthMiddleware::new(service, &*INDEXER_METHODS));
    let module = RpcServer { engine }.into_rpc();

    let handle = Server::builder()
        .set_http_middleware(http_middleware)
        .set_rpc_middleware(rpc_middleware)
        .build(addr.parse::<SocketAddr>()?)
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

        let _ = server
            .initialise(B256Wrapper::new(B256::from_slice(&[1; 32])), 20, 0)
            .await;

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
        let _ = server
            .initialise(B256Wrapper::new(B256::from_slice(&[1; 32])), 20, 0)
            .await;

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
        let _ = server
            .initialise(B256Wrapper::new(B256::from_slice(&[1; 32])), 20, 0)
            .await;

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
                BytesWrapper::empty(),
                20,
                B256Wrapper::new(B256::from_slice(&[1; 32])),
                0,
                None,
                None,
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().unwrap().to.unwrap().address,
            *INVALID_ADDRESS
        );
    }
}
