use std::error::Error;
use std::net::SocketAddr;

use jsonrpsee::core::{async_trait, RpcResult};
use jsonrpsee::server::{RpcServiceBuilder, Server, ServerHandle};
use jsonrpsee::types::{ErrorObject, ErrorObjectOwned};
use revm::primitives::B256;
use tracing::{event, instrument, Level};

use crate::brc20_controller::{
    decode_brc20_balance_result, load_brc20_balance_tx, load_brc20_burn_tx, load_brc20_mint_tx,
};
use crate::db::types::{BlockResponseED, LogResponseED, TxED, TxReceiptED};
use crate::evm::get_evm_address;
use crate::server::api::GetLogsFilter;
use crate::server::server_instance::ServerInstance;
use crate::server::types::TxInfo;
use crate::server::Brc20ProgApiServer;

use super::api::{AddressWrapper, B256Wrapper, BytesWrapper, U256Wrapper};

pub struct RpcServer {
    server_instance: ServerInstance,
}

impl RpcServer {
    fn parse_block_number(&self, number: &str) -> Result<u64, ErrorObject<'static>> {
        if number == "latest" {
            Ok(self.server_instance.get_latest_block_height())
        } else if number.starts_with("0x") {
            u64::from_str_radix(&number[2..], 16)
                .map_err(|_| wrap_error_message("Invalid block number"))
        } else {
            number
                .parse()
                .map_err(|_| wrap_error_message("Invalid block number"))
        }
    }
}

fn wrap_error_message(message: &'static str) -> ErrorObject<'static> {
    event!(Level::ERROR, "Error: {:?}", message);
    RpcServerError::new(message).into()
}

#[async_trait]
impl Brc20ProgApiServer for RpcServer {
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

        self.server_instance
            .add_tx_to_block(
                timestamp,
                &load_brc20_mint_tx(ticker, get_evm_address(&to_pkscript), amount.value()),
                tx_idx,
                self.server_instance.get_latest_block_height() + 1,
                hash.value(),
                inscription_id,
            )
            .map_err(wrap_error_message)
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

        self.server_instance
            .add_tx_to_block(
                timestamp,
                &load_brc20_burn_tx(ticker, get_evm_address(&from_pkscript), amount.value()),
                tx_idx,
                self.server_instance.get_latest_block_height() + 1,
                hash.value(),
                inscription_id,
            )
            .map_err(wrap_error_message)
    }

    #[instrument(skip(self))]
    async fn balance(&self, address_pkscript: String, ticker: String) -> RpcResult<String> {
        event!(Level::INFO, "Checking balance");

        self.server_instance
            .call_contract(&load_brc20_balance_tx(
                ticker,
                get_evm_address(&address_pkscript),
            ))
            .map(|receipt| {
                format!(
                    "0x{:x}",
                    decode_brc20_balance_result(receipt.result_bytes.as_ref())
                )
            })
            .map_err(wrap_error_message)
    }

    async fn block_number(&self) -> RpcResult<String> {
        let height = self.server_instance.get_latest_block_height();
        Ok(format!("0x{:x}", height))
    }

    #[instrument(skip(self))]
    async fn get_block_by_number(&self, block: String) -> RpcResult<BlockResponseED> {
        event!(Level::INFO, "Getting block by number");
        let number = self.parse_block_number(&block)?;
        let block = self.server_instance.get_block_by_number(number);
        if let Some(block) = block {
            Ok(block)
        } else {
            Err(RpcServerError::new("Block not found").into())
        }
    }

    #[instrument(skip(self))]
    async fn get_block_by_hash(&self, block: B256Wrapper) -> RpcResult<BlockResponseED> {
        event!(Level::INFO, "Getting block by number");
        let block = self.server_instance.get_block_by_hash(block.value());
        if let Some(block) = block {
            Ok(block)
        } else {
            Err(RpcServerError::new("Block not found").into())
        }
    }

    #[instrument(skip(self))]
    async fn get_transaction_count(&self, account: String, block: String) -> RpcResult<String> {
        event!(Level::INFO, "Getting transaction count");
        let account = account.parse().unwrap();
        let block = self.parse_block_number(&block)?;
        let count = self.server_instance.get_transaction_count(account, block);
        if count.is_err() {
            return Err(RpcServerError::new("Couldn't get transaction count").into());
        }
        Ok(format!("0x{:x}", count.unwrap()))
    }

    #[instrument(skip(self))]
    async fn get_block_transaction_count_by_number(&self, block: String) -> RpcResult<String> {
        event!(Level::INFO, "Getting block transaction count");
        let block = self.parse_block_number(&block)?;
        let count = self
            .server_instance
            .get_block_transaction_count_by_number(block);
        if count.is_err() {
            return Err(RpcServerError::new("Couldn't get block transaction count").into());
        }
        Ok(format!("0x{:x}", count.unwrap()))
    }

    #[instrument(skip(self))]
    async fn get_block_transaction_count_by_hash(&self, block: B256Wrapper) -> RpcResult<String> {
        event!(Level::INFO, "Getting block transaction count");
        let count = self
            .server_instance
            .get_block_transaction_count_by_hash(block.value());
        if count.is_err() {
            return Err(RpcServerError::new("Couldn't get block transaction count").into());
        }
        Ok(format!("0x{:x}", count.unwrap()))
    }

    #[instrument(skip(self))]
    async fn get_transaction_by_hash(&self, transaction: B256Wrapper) -> RpcResult<Option<TxED>> {
        event!(Level::INFO, "Getting transaction by hash");
        Ok(self
            .server_instance
            .get_transaction_by_hash(transaction.value()))
    }

    #[instrument(skip(self))]
    async fn get_transaction_by_block_hash_and_index(
        &self,
        block_hash: B256Wrapper,
        tx_idx: u64,
    ) -> RpcResult<Option<TxED>> {
        event!(Level::INFO, "Getting transaction by block hash and index");
        Ok(self
            .server_instance
            .get_transaction_by_block_hash_and_index(block_hash.value(), tx_idx))
    }

    #[instrument(skip(self))]
    async fn get_transaction_by_block_number_and_index(
        &self,
        block_number: u64,
        tx_idx: u64,
    ) -> RpcResult<Option<TxED>> {
        event!(Level::INFO, "Getting transaction by block number and index");
        let tx = self
            .server_instance
            .get_transaction_by_block_number_and_index(block_number, tx_idx);
        Ok(tx)
    }

    #[instrument(skip(self))]
    async fn get_transaction_receipt(
        &self,
        transaction: B256Wrapper,
    ) -> RpcResult<Option<TxReceiptED>> {
        event!(Level::INFO, "Getting transaction receipt");
        Ok(self
            .server_instance
            .get_transaction_receipt(transaction.value()))
    }

    async fn get_logs(&self, filter: GetLogsFilter) -> RpcResult<Vec<LogResponseED>> {
        Ok(self.server_instance.get_logs(
            Some(self.parse_block_number(&filter.from_block.unwrap_or("latest".to_string()))?),
            Some(self.parse_block_number(&filter.to_block.unwrap_or("latest".to_string()))?),
            filter.address.map(|x| x.value()),
            filter
                .topics
                .map(|vec| vec.into_iter().map(|topic| topic.value()).collect()),
        ))
    }

    #[instrument(skip(self))]
    async fn mine(&self, block_count: u64, timestamp: u64) -> RpcResult<()> {
        event!(Level::INFO, "Mining empty blocks");
        self.server_instance
            .mine_block(block_count, timestamp, B256::ZERO)
            .map_err(wrap_error_message)
    }

    async fn call(
        &self,
        from: AddressWrapper,
        to: Option<AddressWrapper>,
        data: BytesWrapper,
    ) -> RpcResult<TxReceiptED> {
        self.server_instance
            .call_contract(&TxInfo {
                from: from.value(),
                to: to.map(|x| x.value()),
                data: data.value().clone(),
            })
            .map_err(wrap_error_message)
    }

    async fn estimate_gas(
        &self,
        from: AddressWrapper,
        to: Option<AddressWrapper>,
        data: BytesWrapper,
    ) -> RpcResult<String> {
        Ok(format!(
            "0x{:x}",
            self.server_instance
                .call_contract(&TxInfo {
                    from: from.value(),
                    to: to.map(|x| x.value()),
                    data: data.value().clone(),
                })
                .unwrap()
                .gas_used
        ))
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
            self.server_instance
                .get_storage_at(contract.value(), location.value())
        ))
    }

    #[instrument(skip(self))]
    async fn initialise(
        &self,
        genesis_hash: B256Wrapper,
        genesis_timestamp: u64,
        genesis_height: u64,
    ) -> RpcResult<()> {
        event!(Level::INFO, "Initialising server");
        self.server_instance
            .initialise(genesis_hash.value(), genesis_timestamp, genesis_height)
            .map_err(wrap_error_message)
    }

    #[instrument(skip(self))]
    async fn get_transaction_receipt_by_inscription_id(
        &self,
        inscription_id: String,
    ) -> RpcResult<Option<TxReceiptED>> {
        event!(Level::INFO, "Getting transaction receipt by inscription id");
        let receipt = self
            .server_instance
            .get_transaction_receipt_by_inscription_id(inscription_id);
        Ok(receipt)
    }

    #[instrument(skip(self, data))]
    async fn add_tx_to_block(
        &self,
        from_pkscript: String,
        to: Option<AddressWrapper>,
        data: BytesWrapper,
        timestamp: u64,
        hash: B256Wrapper,
        tx_idx: u64,
        inscription_id: Option<String>,
    ) -> RpcResult<TxReceiptED> {
        event!(Level::INFO, "Adding tx to block");
        self.server_instance
            .add_tx_to_block(
                timestamp,
                &TxInfo {
                    from: get_evm_address(&from_pkscript),
                    to: to.map(|x| x.value()),
                    data: data.value().clone(),
                },
                tx_idx,
                self.server_instance.get_latest_block_height() + 1,
                hash.value(),
                inscription_id,
            )
            .map_err(wrap_error_message)
    }

    #[instrument(skip(self))]
    async fn finalise_block(
        &self,
        timestamp: u64,
        hash: B256Wrapper,
        block_tx_count: u64,
    ) -> RpcResult<()> {
        let block_height = self.server_instance.get_latest_block_height() + 1;
        event!(Level::INFO, "Finalising block {}", block_height);
        self.server_instance
            .finalise_block(timestamp, block_height, hash.value(), block_tx_count)
            .map_err(wrap_error_message)
    }

    #[instrument(skip(self))]
    async fn reorg(&self, latest_valid_block_number: u64) -> RpcResult<()> {
        event!(Level::WARN, "Reorg!");
        self.server_instance
            .reorg(latest_valid_block_number)
            .map_err(wrap_error_message)
    }

    #[instrument(skip(self))]
    async fn commit_to_database(&self) -> RpcResult<()> {
        event!(Level::INFO, "Committing to database");
        self.server_instance
            .commit_to_db()
            .map_err(wrap_error_message)
    }

    #[instrument(skip(self))]
    async fn clear_caches(&self) -> RpcResult<()> {
        event!(Level::INFO, "Clearing caches");
        self.server_instance.clear_caches();
        Ok(())
    }

    #[instrument(skip(self))]
    async fn get_code(&self, contract: AddressWrapper) -> RpcResult<String> {
        event!(Level::INFO, "Getting contract code");
        let result = self.server_instance.get_contract_bytecode(contract.value());
        if let Some(bytecode) = result {
            Ok(hex::encode(bytecode))
        } else {
            Err(RpcServerError::new("Contract bytecode not found").into())
        }
    }
}

struct RpcServerError {
    message: &'static str,
}

impl RpcServerError {
    fn new(message: &'static str) -> Self {
        Self { message }
    }
}

impl Into<ErrorObject<'static>> for RpcServerError {
    fn into(self) -> ErrorObject<'static> {
        ErrorObjectOwned::owned(400, self.message, Option::<()>::None)
    }
}

pub async fn start_rpc_server(
    addr: String,
    server_instance: ServerInstance,
) -> Result<ServerHandle, Box<dyn Error>> {
    let server = Server::builder()
        .set_rpc_middleware(RpcServiceBuilder::new().rpc_logger(1024))
        .build(addr.parse::<SocketAddr>()?)
        .await?;
    let module = RpcServer { server_instance }.into_rpc();
    let handle = server.start(module);

    Ok(handle)
}
