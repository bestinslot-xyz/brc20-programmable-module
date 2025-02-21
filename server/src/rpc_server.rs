use std::{error::Error, net::SocketAddr, str::FromStr};

use db::types::{BlockResponseED, LogResponseED, TxED, TxReceiptED};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    server::{RpcServiceBuilder, Server, ServerHandle},
    types::{ErrorObject, ErrorObjectOwned},
};
use revm::primitives::{Bytes, B256, U256};

use crate::{
    api::GetLogsFilter,
    brc20_controller::{load_brc20_burn_tx, load_brc20_mint_tx},
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

#[async_trait]
impl Brc20ProgApiServer for RpcServer {
    async fn deposit(
        &self,
        from: String,
        ticker: String,
        amount: String,
        timestamp: u64,
        hash: String,
        tx_idx: u64,
    ) -> RpcResult<bool> {
        self.server_instance
            .add_tx_to_block(
                timestamp,
                &load_brc20_mint_tx(
                    ticker,
                    from.parse().unwrap(),
                    U256::from_str(&amount).unwrap(),
                ),
                tx_idx,
                hash.parse().unwrap(),
            )
            .map_err(|e| RpcServerError::new(e).into())
            .map(|receipt| receipt.status == 1)
    }

    async fn withdraw(
        &self,
        to: String,
        ticker: String,
        amount: String,
        timestamp: u64,
        hash: String,
        tx_idx: u64,
    ) -> RpcResult<bool> {
        self.server_instance
            .add_tx_to_block(
                timestamp,
                &load_brc20_burn_tx(
                    ticker,
                    to.parse().unwrap(),
                    U256::from_str(&amount).unwrap(),
                ),
                tx_idx,
                hash.parse().unwrap(),
            )
            .map_err(|e| RpcServerError::new(e).into())
            .map(|receipt| receipt.status == 1)
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

    async fn mine(&self, block_cnt: u64, timestamp: u64) -> RpcResult<()> {
        let hash = B256::ZERO;
        self.server_instance
            .mine_block(block_cnt, timestamp, hash)
            .map_err(|e| RpcServerError::new(e).into())
    }

    async fn call(&self, from: String, to: Option<String>, data: String) -> RpcResult<TxReceiptED> {
        let from = from.parse().unwrap();
        let to = to.map(|x| x.parse().unwrap());
        let data = hex::decode(data).unwrap();
        let data = Bytes::from(data);
        let tx_info = TxInfo { from, to, data };
        self.server_instance
            .call_contract(&tx_info)
            .map_err(|e| RpcServerError::new(e).into())
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

    async fn add_tx_to_block(
        &self,
        from: String,
        to: Option<String>,
        data: String,
        timestamp: u64,
        hash: String,
        tx_idx: u64,
    ) -> RpcResult<TxReceiptED> {
        let from = from.parse().unwrap();
        let to = to.map(|x| x.parse().unwrap());
        let data = hex::decode(data).unwrap();
        let data = Bytes::from(data);
        let hash = hash.parse().unwrap();
        let tx_info = TxInfo { from, to, data };
        self.server_instance
            .add_tx_to_block(timestamp, &tx_info, tx_idx, hash)
            .map_err(|e| RpcServerError::new(e).into())
    }

    async fn finalise_block(
        &self,
        timestamp: u64,
        hash: String,
        block_tx_cnt: u64,
    ) -> RpcResult<()> {
        self.server_instance
            .finalise_block(timestamp, hash.parse().unwrap(), block_tx_cnt, None)
            .map_err(|e| RpcServerError::new(e).into())
    }

    async fn finalise_block_with_txes(
        &self,
        timestamp: u64,
        hash: String,
        txes: Vec<TxInfo>,
    ) -> RpcResult<Vec<TxReceiptED>> {
        self.server_instance
            .finalise_block_with_txes(timestamp, hash.parse().unwrap(), txes)
            .map_err(|e| RpcServerError::new(e).into())
    }

    async fn reorg(&self, latest_valid_block_number: u64) -> RpcResult<()> {
        self.server_instance
            .reorg(latest_valid_block_number)
            .map_err(|e| RpcServerError::new(e).into())
    }

    async fn commit_to_database(&self) -> RpcResult<()> {
        self.server_instance
            .commit_to_db()
            .map_err(|e| RpcServerError::new(e).into())
    }

    async fn clear_caches(&self) -> RpcResult<()> {
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
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::TRACE)
            .finish(),
    )?;
    let server = Server::builder()
        .set_rpc_middleware(RpcServiceBuilder::new().rpc_logger(1024))
        .build(addr.parse::<SocketAddr>()?)
        .await?;
    let module = RpcServer { server_instance }.into_rpc();
    let handle = server.start(module);

    Ok(handle)
}
