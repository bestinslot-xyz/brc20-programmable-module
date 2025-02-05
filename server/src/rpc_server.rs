use std::{error::Error, net::SocketAddr};

use jsonrpsee::server::{RpcModule, Server, ServerHandle};
use revm::primitives::{Bytes, B256};
use serde_json::Value;

use crate::{
    server_instance::ServerInstance,
    types::{get_serializeable_execution_result, BlockResJSON, TxInfo},
};

pub async fn start_rpc_server(
    addr: &str,
    server_instance: ServerInstance,
) -> Result<ServerHandle, Box<dyn Error>> {
    let server = Server::builder()
        .build(addr.parse::<SocketAddr>()?)
        .await?;

    let mut module = RpcModule::new(server_instance);
    module.register_method("custom_blockNumber", |_, ctx, _| {
        ctx.get_latest_block_height().to_string()
    })?;
    module.register_method("custom_getBlockByNumber", |params, ctx, _| {
        let number = params.parse::<GetBlockByNumberRequest>().unwrap().number;
        let block = ctx.get_block_by_number(number);
        if block.is_none() {
            return Value::Null;
        }
        let block = block.unwrap();
        serde_json::json!(BlockResJSON {
            number: block.number.to_string(),
            timestamp: block.timestamp.to_string(),
            gas_used: block.gas_used.to_string(),
            mine_tm: block.mine_tm.to_string(),
            hash: format!("{:?}", block.hash),
        })
    })?;
    module.register_method("custom_mine", |params, ctx, _| {
        let MineRequest { block_cnt, timestamp } = params.parse().unwrap();
        let hash = B256::ZERO;
        let result = ctx.mine_block(block_cnt, timestamp, hash);

        if result.is_err() {
            return Value::Null;
        }
        Value::Null
    })?;
    module.register_method("custom_call", |params, ctx, _| {
        let CallRequest { from, to, data } = params.parse::<CallRequest>().unwrap();
        let from = from.parse().unwrap();
        let to = to.map(|x| x.parse().unwrap());
        let data = hex::decode(data).unwrap();
        let data = Bytes::from(data);
        let tx_info = TxInfo { from, to, data };
        let result = ctx.call_contract(&tx_info);

        if result.is_err() {
            return Value::Null;
        }
        let (result, nonce, txhash) = result.unwrap();
        serde_json::json!(get_serializeable_execution_result(result, txhash, nonce))
    })?;
    module.register_method("custom_addTxToBlock", |params, ctx, _| {
        let AddTxToBlockRequest {
            from,
            to,
            data,
            timestamp,
            hash,
            tx_idx,
        } = params.parse().unwrap();
        let from = from.parse().unwrap();
        let to = to.map(|x| x.parse().unwrap());
        let data = hex::decode(data).unwrap();
        let data = Bytes::from(data);
        let hash = hash.parse().unwrap();
        let tx_info = TxInfo { from, to, data };

        let result = ctx.add_tx_to_block(timestamp, &tx_info, tx_idx, hash);

        if result.is_err() {
            return Value::Null;
        }
        let (result, nonce, txhash) = result.unwrap();
        serde_json::json!(get_serializeable_execution_result(result, txhash, nonce))
    })?;
    module.register_method("custom_finaliseBlock", |params, ctx, _| {
        let FinaliseBlockRequest {
            timestamp,
            hash,
            block_tx_cnt,
        } = params.parse().unwrap();
        let _ = ctx.finalise_block(timestamp, hash.parse().unwrap(), block_tx_cnt, None);
        return Value::Null;
    })?;
    module.register_method("custom_finaliseBlockWithTxes", |params, ctx, _| {
        let FinaliseBlockWithTxesRequest {
            timestamp,
            hash,
            txes,
        } = params.parse().unwrap();
        let result = ctx.finalise_block_with_txes(timestamp, hash.parse().unwrap(), txes);
        if result.is_err() {
            return Value::Null;
        }
        serde_json::json!(result.unwrap())
    })?;
    module.register_method("custom_reorg", |params, ctx, _| {
        let latest_valid_block_number = params
            .parse::<ReorgRequest>()
            .unwrap()
            .latest_valid_block_number;
        let _ = ctx.reorg(latest_valid_block_number);
        return Value::Null;
    })?;
    module.register_method("custom_commitToDb", |_, ctx, _| {
        let _ = ctx.commit_to_db();
        return Value::Null;
    })?;
    module.register_method("custom_clearCaches", |_, ctx, _| {
        ctx.clear_caches();
        return Value::Null;
    })?;
    module.register_method("custom_getContractBytecode", |params, ctx, _| {
        let addr = params
            .parse::<GetContractBytecodeRequest>()
            .unwrap()
            .address
            .parse()
            .unwrap();
        let result = ctx.get_contract_bytecode(addr);

        if result.is_none() {
            return Value::Null;
        }
        let bytecode = result.unwrap();
        serde_json::json!(&hex::encode(bytecode))
    })?;

    let handle = server.start(module);

    Ok(handle)
}

#[derive(serde::Deserialize)]
pub struct CallRequest {
    pub from: String,
    pub to: Option<String>,
    pub data: String,
}

#[derive(serde::Deserialize)]
pub struct GetBlockByNumberRequest {
    pub number: u64,
}

#[derive(serde::Deserialize)]
pub struct GetContractBytecodeRequest {
    pub address: String,
}

#[derive(serde::Deserialize)]
pub struct AddTxToBlockRequest {
    pub from: String,
    pub to: Option<String>,
    pub data: String,
    pub timestamp: u64,
    pub hash: String,
    pub tx_idx: u64,
}

#[derive(serde::Deserialize)]
pub struct FinaliseBlockRequest {
    pub timestamp: u64,
    pub hash: String,
    pub block_tx_cnt: u64,
}

#[derive(serde::Deserialize)]
pub struct FinaliseBlockWithTxesRequest {
    pub timestamp: u64,
    pub hash: String,
    pub txes: Vec<Value>,
}

#[derive(serde::Deserialize)]
pub struct ReorgRequest {
    pub latest_valid_block_number: u64,
}

#[derive(serde::Deserialize)]
pub struct MineRequest {
    pub block_cnt: u64,
    pub timestamp: u64,
}
