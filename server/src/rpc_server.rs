use std::{error::Error, net::SocketAddr, str::FromStr};

use db::types::{BlockResponseED, LogResponseED, TxED, TxReceiptED};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    server::{RpcServiceBuilder, Server, ServerHandle},
    types::{ErrorObject, ErrorObjectOwned},
};
use revm::primitives::{Bytes, B256, U256};
use tracing::{event, instrument};

use crate::{
    api::GetLogsFilter,
    brc20_controller::{
        decode_brc20_balance_result, load_brc20_balance_tx, load_brc20_burn_tx, load_brc20_mint_tx,
    },
    evm::get_evm_address,
    server_instance::ServerInstance,
    types::TxInfo,
    Brc20ProgApiServer,
};

pub struct RpcServer {
    server_instance: ServerInstance,
}

impl RpcServer {
    fn parse_block_number(&self, number: &str) -> u64 {
        if number == "latest" {
            self.server_instance.get_latest_block_height()
        } else if number.starts_with("0x") {
            u64::from_str_radix(&number[2..], 16).unwrap()
        } else {
            number.parse().unwrap()
        }
    }
}

fn wrap_error_message(message: &'static str) -> ErrorObject<'static> {
    event!(tracing::Level::ERROR, "Error: {:?}", message);
    RpcServerError::new(message).into()
}

#[async_trait]
impl Brc20ProgApiServer for RpcServer {
    #[instrument(skip(self))]
    async fn deposit(
        &self,
        to_pkscript: String,
        ticker: String,
        amount: String,
        timestamp: u64,
        hash: String,
        tx_idx: u64,
    ) -> RpcResult<bool> {
        event!(tracing::Level::DEBUG, "Depositing");

        self.server_instance
            .add_tx_to_block(
                timestamp,
                &load_brc20_mint_tx(
                    ticker,
                    get_evm_address(&to_pkscript),
                    U256::from_str(&amount).unwrap(),
                ),
                tx_idx,
                self.server_instance.get_latest_block_height() + 1,
                hash.parse().unwrap(),
            )
            .map_err(wrap_error_message)
            .map(|receipt| receipt.status == 1)
    }

    #[instrument(skip(self))]
    async fn withdraw(
        &self,
        from_pkscript: String,
        ticker: String,
        amount: String,
        timestamp: u64,
        hash: String,
        tx_idx: u64,
    ) -> RpcResult<bool> {
        event!(tracing::Level::DEBUG, "Withdrawing");

        self.server_instance
            .add_tx_to_block(
                timestamp,
                &load_brc20_burn_tx(
                    ticker,
                    get_evm_address(&from_pkscript),
                    U256::from_str(&amount).unwrap(),
                ),
                tx_idx,
                self.server_instance.get_latest_block_height() + 1,
                hash.parse().unwrap(),
            )
            .map_err(wrap_error_message)
            .map(|receipt| receipt.status == 1)
    }

    #[instrument(skip(self))]
    async fn balance(&self, address_pkscript: String, ticker: String) -> RpcResult<String> {
        event!(tracing::Level::DEBUG, "Checking balance");

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

    async fn get_block_by_number(&self, block: String) -> RpcResult<BlockResponseED> {
        let number = self.parse_block_number(&block);
        let block = self.server_instance.get_block_by_number(number);
        if let Some(block) = block {
            Ok(block)
        } else {
            Err(RpcServerError::new("Block not found").into())
        }
    }

    async fn get_block_by_hash(&self, block: String) -> RpcResult<BlockResponseED> {
        let block = self
            .server_instance
            .get_block_by_hash(block.parse().unwrap());
        if let Some(block) = block {
            Ok(block)
        } else {
            Err(RpcServerError::new("Block not found").into())
        }
    }

    async fn get_transaction_count(&self, account: String, block: String) -> RpcResult<String> {
        let account = account.parse().unwrap();
        let block = self.parse_block_number(&block);
        let count = self.server_instance.get_transaction_count(account, block);
        if count.is_err() {
            return Err(RpcServerError::new("Couldn't get transaction count").into());
        }
        Ok(format!("0x{:x}", count.unwrap()))
    }

    async fn get_block_transaction_count_by_number(&self, block: String) -> RpcResult<String> {
        let block = self.parse_block_number(&block);
        let count = self
            .server_instance
            .get_block_transaction_count_by_number(block);
        if count.is_err() {
            return Err(RpcServerError::new("Couldn't get block transaction count").into());
        }
        Ok(format!("0x{:x}", count.unwrap()))
    }

    async fn get_block_transaction_count_by_hash(&self, block: String) -> RpcResult<String> {
        let block_hash = B256::from_str(&block[2..]).unwrap();
        let count = self
            .server_instance
            .get_block_transaction_count_by_hash(block_hash);
        if count.is_err() {
            return Err(RpcServerError::new("Couldn't get block transaction count").into());
        }
        Ok(format!("0x{:x}", count.unwrap()))
    }

    async fn get_transaction_by_hash(&self, transaction: String) -> RpcResult<Option<TxED>> {
        let tx_hash = B256::from_str(&transaction[2..]).unwrap();
        let tx = self.server_instance.get_transaction_by_hash(tx_hash);
        Ok(tx)
    }

    async fn get_transaction_by_block_hash_and_index(
        &self,
        block_hash: String,
        tx_idx: u64,
    ) -> RpcResult<Option<TxED>> {
        let block_hash = B256::from_str(&block_hash[2..]).unwrap();
        let tx = self
            .server_instance
            .get_transaction_by_block_hash_and_index(block_hash, tx_idx);
        Ok(tx)
    }

    async fn get_transaction_by_block_number_and_index(
        &self,
        block_number: u64,
        tx_idx: u64,
    ) -> RpcResult<Option<TxED>> {
        let tx = self
            .server_instance
            .get_transaction_by_block_number_and_index(block_number, tx_idx);
        Ok(tx)
    }

    async fn get_transaction_receipt(&self, transaction: String) -> RpcResult<Option<TxReceiptED>> {
        let tx_hash = B256::from_str(&transaction[2..]).unwrap();
        let receipt = self.server_instance.get_transaction_receipt(tx_hash);
        Ok(receipt)
    }

    async fn get_logs(&self, filter: GetLogsFilter) -> RpcResult<Vec<LogResponseED>> {
        Ok(self.server_instance.get_logs(
            Some(self.parse_block_number(&filter.from_block.unwrap_or("latest".to_string()))),
            Some(self.parse_block_number(&filter.to_block.unwrap_or("latest".to_string()))),
            filter.address.map(|x| x.parse().unwrap()),
            filter
                .topics
                .map(|x| x.into_iter().map(|y| y.parse().unwrap()).collect()),
        ))
    }

    #[instrument(skip(self))]
    async fn mine(&self, block_cnt: u64, timestamp: u64) -> RpcResult<()> {
        let hash = B256::ZERO;
        self.server_instance
            .mine_block(block_cnt, timestamp, hash)
            .map_err(wrap_error_message)
    }

    async fn call(
        &self,
        from_pkscript: String,
        to: Option<String>,
        mut data: String,
    ) -> RpcResult<TxReceiptED> {
        let from = get_evm_address(&from_pkscript);
        let to = to.map(|x| x.parse().unwrap());
        if data.starts_with("0x") {
            data = data[2..].to_string();
        }
        let data = hex::decode(data).unwrap();
        let data = Bytes::from(data);
        let tx_info = TxInfo { from, to, data };
        self.server_instance
            .call_contract(&tx_info)
            .map_err(wrap_error_message)
    }

    async fn estimate_gas(
        &self,
        from: String,
        to: Option<String>,
        data: String,
    ) -> RpcResult<String> {
        let from = from.parse().unwrap();
        let to = to.map(|x| x.parse().unwrap());
        let data = hex::decode(data).unwrap();
        let data = Bytes::from(data);
        let tx_info = TxInfo { from, to, data };
        let gas_used = self
            .server_instance
            .call_contract(&tx_info)
            .unwrap()
            .gas_used;
        Ok(format!("0x{:x}", gas_used))
    }

    async fn get_storage_at(&self, contract: String, location: String) -> RpcResult<String> {
        let addr = contract.parse().unwrap();
        let location = location.parse().unwrap();
        let storage = self.server_instance.get_storage_at(addr, location);
        Ok(format!("0x{:x}", storage))
    }

    #[instrument(skip(self))]
    async fn initialise(&self, genesis_hash: String, genesis_timestamp: u64) -> RpcResult<()> {
        event!(tracing::Level::INFO, "Initialising server");
        self.server_instance
            .initialise(genesis_hash.parse().unwrap(), genesis_timestamp)
            .map_err(wrap_error_message)
    }

    #[instrument(skip(self, data))]
    async fn add_tx_to_block(
        &self,
        from_pkscript: String,
        to: Option<String>,
        mut data: String,
        timestamp: u64,
        hash: String,
        tx_idx: u64,
    ) -> RpcResult<TxReceiptED> {
        event!(tracing::Level::INFO, "Adding tx to block");
        let from = get_evm_address(&from_pkscript);
        let to = to.map(|x| x.parse().unwrap());
        if data.starts_with("0x") {
            data = data[2..].to_string();
        }
        let data = hex::decode(data).unwrap();
        let data = Bytes::from(data);
        let hash = hash.parse().unwrap();
        let tx_info = TxInfo { from, to, data };

        self.server_instance
            .add_tx_to_block(
                timestamp,
                &tx_info,
                tx_idx,
                self.server_instance.get_latest_block_height() + 1,
                hash,
            )
            .map_err(wrap_error_message)
    }

    #[instrument(skip(self))]
    async fn finalise_block(
        &self,
        timestamp: u64,
        hash: String,
        block_tx_cnt: u64,
    ) -> RpcResult<()> {
        event!(tracing::Level::INFO, "Finalising block");
        self.server_instance
            .finalise_block(
                timestamp,
                self.server_instance.get_latest_block_height() + 1,
                B256::from_str(if hash.starts_with("0x") {
                    &hash[2..]
                } else {
                    &hash
                })
                .unwrap(),
                block_tx_cnt,
            )
            .map_err(wrap_error_message)
    }

    #[instrument(skip(self, txes))]
    async fn finalise_block_with_txes(
        &self,
        timestamp: u64,
        hash: String,
        txes: Vec<TxInfo>,
    ) -> RpcResult<Vec<TxReceiptED>> {
        event!(tracing::Level::INFO, "Finalising block with txes");
        self.server_instance
            .finalise_block_with_txes(
                timestamp,
                self.server_instance.get_latest_block_height() + 1,
                hash.parse().unwrap(),
                txes,
            )
            .map_err(wrap_error_message)
    }

    #[instrument(skip(self))]
    async fn reorg(&self, latest_valid_block_number: u64) -> RpcResult<()> {
        event!(tracing::Level::WARN, "Reorg!");
        self.server_instance
            .reorg(latest_valid_block_number)
            .map_err(wrap_error_message)
    }

    #[instrument(skip(self))]
    async fn commit_to_database(&self) -> RpcResult<()> {
        event!(tracing::Level::INFO, "Committing to database");
        self.server_instance
            .commit_to_db()
            .map_err(wrap_error_message)
    }

    #[instrument(skip(self))]
    async fn clear_caches(&self) -> RpcResult<()> {
        event!(tracing::Level::INFO, "Clearing caches");
        self.server_instance.clear_caches();
        Ok(())
    }

    async fn get_code(&self, contract: String) -> RpcResult<String> {
        let addr = contract.parse().unwrap();
        let result = self.server_instance.get_contract_bytecode(addr);
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
    addr: &str,
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
