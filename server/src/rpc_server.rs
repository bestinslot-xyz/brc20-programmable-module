use std::{error::Error, net::SocketAddr};

use jsonrpsee::{
    core::{async_trait, RpcResult},
    server::{Server, ServerHandle},
    types::{ErrorObject, ErrorObjectOwned},
};
use revm::primitives::{Bytes, B256};

use crate::{
    server_instance::ServerInstance,
    types::{BlockResJSON, SerializableExecutionResult, TxInfo},
    Brc20ProgApiServer,
};

pub struct RpcServer {
    server_instance: ServerInstance,
}

#[async_trait]
impl Brc20ProgApiServer for RpcServer {
    async fn block_number(&self) -> RpcResult<u64> {
        Ok(self.server_instance.get_latest_block_height())
    }

    async fn get_block_by_number(&self, number: u64) -> RpcResult<BlockResJSON> {
        let block = self.server_instance.get_block_by_number(number);
        if let Some(block) = block {
            Ok(BlockResJSON {
                number: block.number.to_string(),
                timestamp: block.timestamp.to_string(),
                gas_used: block.gas_used.to_string(),
                mine_tm: block.mine_tm.to_string(),
                hash: format!("{:?}", block.hash),
            })
        } else {
            Err(RpcServerError::new("Block not found").into())
        }
    }

    async fn get_block_by_hash(&self, hash: String) -> RpcResult<BlockResJSON> {
        let block = self
            .server_instance
            .get_block_by_hash(hash.parse().unwrap());
        if let Some(block) = block {
            Ok(BlockResJSON {
                number: block.number.to_string(),
                timestamp: block.timestamp.to_string(),
                gas_used: block.gas_used.to_string(),
                mine_tm: block.mine_tm.to_string(),
                hash: format!("{:?}", block.hash),
            })
        } else {
            Err(RpcServerError::new("Block not found").into())
        }
    }

    async fn mine(&self, block_cnt: u64, timestamp: u64) -> RpcResult<()> {
        let hash = B256::ZERO;
        self.server_instance
            .mine_block(block_cnt, timestamp, hash)
            .map_err(|e| RpcServerError::new(e).into())
    }

    async fn call(
        &self,
        from: String,
        to: Option<String>,
        data: String,
    ) -> RpcResult<SerializableExecutionResult> {
        let from = from.parse().unwrap();
        let to = to.map(|x| x.parse().unwrap());
        let data = hex::decode(data).unwrap();
        let data = Bytes::from(data);
        let tx_info = TxInfo { from, to, data };
        self.server_instance
            .call_contract(&tx_info)
            .map_err(|e| RpcServerError::new(e).into())
    }

    async fn add_tx_to_block(
        &self,
        from: String,
        to: Option<String>,
        data: String,
        timestamp: u64,
        hash: String,
        tx_idx: u64,
    ) -> RpcResult<SerializableExecutionResult> {
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
    ) -> RpcResult<Vec<SerializableExecutionResult>> {
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

    async fn get_code(&self, address: String) -> RpcResult<String> {
        let addr = address.parse().unwrap();
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
    let server = Server::builder().build(addr.parse::<SocketAddr>()?).await?;

    let module = RpcServer { server_instance }.into_rpc();
    let handle = server.start(module);

    Ok(handle)
}
