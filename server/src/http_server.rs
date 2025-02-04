use revm::primitives::alloy_primitives::Bytes;
use revm::primitives::B256;

use rouille::try_or_400;
use rouille::Response;

use serde_json::Value;

use crate::server_instance::ServerInstance;
use crate::types::{get_serializeable_execution_result, BlockResJSON, TxInfo};

pub fn start_http_server(instance: ServerInstance) {
    println!("Starting server!");
    rouille::start_server("127.0.0.1:18545", move |request| {
        if request.method() != "POST" {
            return Response::text("Send POST Req!!!").with_status_code(400);
        }
        let json: Value = try_or_400!(rouille::input::json_input(request));
        let method = json.get("method").unwrap().as_str().unwrap();
        let params = json.get("params").unwrap().as_object().unwrap().clone();
        if method == "custom_blockNumber" {
            Response::json(&instance.get_latest_block_height().to_string())
        } else if method == "custom_mine" {
            let block_cnt = params.get("block_cnt").unwrap().as_u64().unwrap();
            let timestamp = params.get("timestamp").unwrap().as_u64().unwrap();
            let hash = B256::ZERO;

            let result = instance.mine_block(block_cnt, timestamp, hash);
            if result.is_err() {
                Response::text(result.unwrap_err().to_string()).with_status_code(400)
            } else {
                Response::json::<Value>(&Value::Null)
            }
        } else if method == "custom_getBlockByNumber" {
            let number = params.get("number").unwrap().as_u64().unwrap();

            if let Some(block) = instance.get_block_by_number(number) {
                Response::json(&BlockResJSON {
                    number: block.number.to_string(),
                    timestamp: block.timestamp.to_string(),
                    gas_used: block.gas_used.to_string(),
                    mine_tm: block.mine_tm.to_string(),
                    hash: format!("{:?}", block.hash),
                })
            } else {
                Response::json::<Value>(&Value::Null)
            }
        } else if method == "custom_call" {
            let from = params
                .get("from")
                .unwrap()
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let to = params
                .get("to")
                .unwrap()
                .as_str()
                .map(|x| x.parse().unwrap());
            let data = Bytes::from(
                hex::decode(
                    params
                        .get("data")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string()
                        .split_at(2)
                        .1,
                )
                .unwrap(),
            );
            let txinfo = TxInfo { from, to, data };

            let result = instance.call_contract(&txinfo);
            if result.is_err() {
                return Response::text(result.unwrap_err().to_string()).with_status_code(400);
            } else {
                let (result, nonce, txhash) = result.unwrap();
                let serializeable_res = get_serializeable_execution_result(result, txhash, nonce);

                Response::json(&serializeable_res)
            }
        } else if method == "get_contract_bytecode" {
            let addr = params
                .get("addr")
                .unwrap()
                .as_str()
                .unwrap()
                .parse()
                .unwrap();

            let bytecode = instance.get_contract_bytecode(addr);
            if bytecode.is_none() {
                return Response::json::<Value>(&Value::Null);
            } else {
                return Response::json(&hex::encode(bytecode.unwrap()));
            }
        } else if method == "custom_addTxToBlock" {
            let timestamp = params.get("timestamp").unwrap().as_u64().unwrap();
            let hash = B256::from_slice(
                hex::decode(
                    params
                        .get("hash")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string()
                        .split_at(2)
                        .1,
                )
                .unwrap()
                .as_slice(),
            );
            let tx_idx = params.get("tx_idx").unwrap().as_u64().unwrap();
            let from = params
                .get("from")
                .unwrap()
                .as_str()
                .unwrap()
                .parse()
                .unwrap();
            let to = params
                .get("to")
                .unwrap()
                .as_str()
                .map(|x| x.parse().unwrap());
            let data = Bytes::from(
                hex::decode(
                    params
                        .get("data")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string()
                        .split_at(2)
                        .1,
                )
                .unwrap(),
            );
            let txinfo = TxInfo { from, to, data };

            let result = instance.add_tx_to_block(timestamp, &txinfo, tx_idx, hash);
            if result.is_err() {
                return Response::text(result.unwrap_err()).with_status_code(400);
            } else {
                let (result, nonce, txhash) = result.unwrap();
                let serializeable_res = get_serializeable_execution_result(result, txhash, nonce);
                Response::json(&serializeable_res)
            }
        } else if method == "custom_finaliseBlock" {
            let timestamp = params.get("timestamp").unwrap().as_u64().unwrap();
            let hash = B256::from_slice(
                hex::decode(
                    params
                        .get("hash")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string()
                        .split_at(2)
                        .1,
                )
                .unwrap()
                .as_slice(),
            );
            let block_tx_cnt = params.get("block_tx_cnt").unwrap().as_u64().unwrap();

            let start_time = std::time::Instant::now();
            let result = instance.finalise_block(timestamp, hash, block_tx_cnt, start_time);

            if result.is_err() {
                return Response::text(result.unwrap_err()).with_status_code(400);
            } else {
                Response::json::<Value>(&Value::Null)
            }
        } else if method == "custom_finaliseBlockWithTxes" {
            let timestamp = params.get("timestamp").unwrap().as_u64().unwrap();
            let hash = B256::from_slice(
                hex::decode(
                    params
                        .get("hash")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string()
                        .split_at(2)
                        .1,
                )
                .unwrap()
                .as_slice(),
            );
            let txes = params.get("txes").unwrap().as_array().unwrap();

            let result = instance.finalise_block_with_txes(timestamp, hash, txes.to_vec());
            if result.is_err() {
                return Response::text(result.err().unwrap()).with_status_code(400);
            } else {
                Response::json(&result.unwrap())
            }
        } else if method == "custom_commit_to_db" {
            let result = instance.commit_to_db();
            if result.is_err() {
                return Response::text(result.unwrap_err()).with_status_code(400);
            } else {
                Response::json::<Value>(&Value::Null)
            }
        } else if method == "custom_reorg" {
            let latest_valid_block_number = params
                .get("latest_valid_block_number")
                .unwrap()
                .as_u64()
                .unwrap();

            let result = instance.reorg(latest_valid_block_number);
            if result.is_err() {
                return Response::text(result.unwrap_err()).with_status_code(400);
            } else {
                Response::json::<Value>(&Value::Null)
            }
        } else if method == "clear_caches" {
            instance.clear_caches();

            Response::json::<Value>(&Value::Null)
        } else {
            Response::text("Unknown Command").with_status_code(400)
        }
    });
}
