use revm::primitives::env::{ TransactTo, BlockEnv };
use revm::primitives::{ CreateScheme, ExecutionResult, B256, U256 };
use revm::primitives::alloy_primitives::Bytes;

use revm::Database;
use rouille::Response;
use rouille::try_or_400;

use serde_json::Value;

use std::sync::Mutex;

use db::DB;

mod types;
use types::{ BlockResJSON, BlockRes, TxInfo, get_serializeable_execution_result, get_tx_hash };

mod evm;
use evm::{ get_evm, modify_evm_with_tx_env };

pub fn start_server(db_mutex: Mutex<DB>) {
  println!("Starting server!");
  let waiting_tx_cnt_mutex: Mutex<u64> = Mutex::new(0);
  let last_ts_mutex: Mutex<U256> = Mutex::new(U256::ZERO);
  let last_block_hash_mutex: Mutex<B256> = Mutex::new(B256::ZERO);
  let last_block_gas_used_mutex: Mutex<u64> = Mutex::new(0);

  rouille::start_server("127.0.0.1:18545", move |request| {
    if request.method() == "POST" {
      let json: Value = try_or_400!(rouille::input::json_input(request));
      let method = json.get("method").unwrap().as_str().unwrap();
      let params = json.get("params").unwrap().as_object().unwrap().clone();
      if method == "custom_blockNumber" {
        let db = db_mutex.lock().unwrap();

        Response::json(&get_latest_block_height(&db).to_string())
      } else if method == "custom_mine" {
        let waiting_tx_cnt = waiting_tx_cnt_mutex.lock().unwrap();
        if *waiting_tx_cnt != 0 {
          Response::text("there are waiting txes committed to db, cannot mine empty block!!").with_status_code(400)
        } else {
          let db = db_mutex.lock().unwrap();

          let block_cnt = params.get("block_cnt").unwrap().as_u64().unwrap();
          let timestamp = params.get("timestamp").unwrap().as_u64().unwrap();
          let hash = B256::ZERO;

          mine_block(&db, block_cnt, U256::from(timestamp), hash);

          Response::json::<Value>(&Value::Null)
        }
      } else if method == "custom_getBlockByNumber" {
        let db = db_mutex.lock().unwrap();

        let number = U256::from(params.get("number").unwrap().as_u64().unwrap());
        
        if let Some(block) = get_block_by_number(&db, number) {
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
        let waiting_tx_cnt = waiting_tx_cnt_mutex.lock().unwrap();

        if *waiting_tx_cnt != 0 {
          Response::text("there are waiting txes committed to db, cannot use call!!").with_status_code(400)
        } else {
          let from = params.get("from").unwrap().as_str().unwrap().parse().unwrap();
          let to = params.get("to").unwrap().as_str().map(|x| x.parse().unwrap());
          let data = Bytes::from(hex::decode(params.get("data").unwrap().as_str().unwrap().to_string().split_at(2).1).unwrap());
          let txinfo = TxInfo { from, to, data };

          let (result, nonce, txhash) = call_contract(&db_mutex, &txinfo);

          let serializeable_res = get_serializeable_execution_result(result, txhash, nonce);
          
          Response::json(&serializeable_res)
        }
      } else if method == "get_contract_bytecode" {
        let mut db = db_mutex.lock().unwrap();

        let addr = params.get("addr").unwrap().as_str().unwrap().parse().unwrap();
        
        let acct = db.basic(addr).unwrap().unwrap();
        let bytecode = db.read_from_code_map(acct.code_hash).unwrap().unwrap();
        
        Response::json(&hex::encode(bytecode.bytes()))
      } else if method == "custom_addTxToBlock" {
        let timestamp = U256::from(params.get("timestamp").unwrap().as_u64().unwrap());
        let hash = B256::from_slice(hex::decode(params.get("hash").unwrap().as_str().unwrap().to_string().split_at(2).1).unwrap().as_slice());
        let tx_idx = params.get("tx_idx").unwrap().as_u64().unwrap();

        let mut waiting_tx_cnt = waiting_tx_cnt_mutex.lock().unwrap();
        let mut last_ts = last_ts_mutex.lock().unwrap();
        let mut last_block_hash = last_block_hash_mutex.lock().unwrap();
        let mut last_block_gas_used = last_block_gas_used_mutex.lock().unwrap();

        if *waiting_tx_cnt != tx_idx {
          return Response::text("tx_idx is different from waiting tx cnt in block!!").with_status_code(400);
        }
        if *waiting_tx_cnt != 0 {
          if timestamp != *last_ts {
            return Response::text("timestamp is different from other txes in block!!").with_status_code(400);
          }

          if hash != *last_block_hash {
            return Response::text("block hash is different from other txes in block!!").with_status_code(400);
          }
        } else {
          *last_ts = timestamp;
          *last_block_hash = hash;
          *last_block_gas_used = 0;
        }

        let from = params.get("from").unwrap().as_str().unwrap().parse().unwrap();
        let to = params.get("to").unwrap().as_str().map(|x| x.parse().unwrap());
        let data = Bytes::from(hex::decode(params.get("data").unwrap().as_str().unwrap().to_string().split_at(2).1).unwrap());
        let txinfo = TxInfo { from, to, data };

        let (result, nonce, txhash) = add_tx_to_block(&db_mutex, timestamp, &txinfo);
        *waiting_tx_cnt += 1;
        *last_block_gas_used += result.gas_used();

        let serializeable_res = get_serializeable_execution_result(result, txhash, nonce);

        Response::json(&serializeable_res)
      } else if method == "custom_finaliseBlock" {
        let mut waiting_tx_cnt = waiting_tx_cnt_mutex.lock().unwrap();
        let mut last_ts = last_ts_mutex.lock().unwrap();
        let mut last_block_hash = last_block_hash_mutex.lock().unwrap();
        let mut last_block_gas_used = last_block_gas_used_mutex.lock().unwrap();
        let db = db_mutex.lock().unwrap();

        let timestamp = U256::from(params.get("timestamp").unwrap().as_u64().unwrap());
        let hash = B256::from_slice(hex::decode(params.get("hash").unwrap().as_str().unwrap().to_string().split_at(2).1).unwrap().as_slice());
        let block_tx_cnt = params.get("block_tx_cnt").unwrap().as_u64().unwrap();

        if *waiting_tx_cnt != 0 {
          if timestamp != *last_ts {
            return Response::text("timestamp is different from other txes in block!!").with_status_code(400);
          }

          if hash != *last_block_hash {
            return Response::text("block hash is different from other txes in block!!").with_status_code(400);
          }
        } else {
          *last_block_gas_used = 0; // not needed but just for sanity
        }
        if *waiting_tx_cnt != block_tx_cnt {
          return Response::text("block tx cnt is different from waiting tx cnt for block!!").with_status_code(400);
        }

        finalise_block(&db, timestamp, hash, *last_block_gas_used, block_tx_cnt);

        *waiting_tx_cnt = 0;
        *last_ts = U256::ZERO;
        *last_block_hash = B256::ZERO;
        *last_block_gas_used = 0;

        Response::json::<Value>(&Value::Null)
      } else {
        Response::text("Faulty Command").with_status_code(400)
      }
    } else {
      Response::text("Send POST Req!!!").with_status_code(400)
    }
  });
}


fn get_block_by_number(db: &DB, number: U256) -> Option<BlockRes> {
  println!("Getting block by number {}", number);
  let block_hash = db.read_from_block_hashes(number).unwrap();
  if block_hash.is_none() { return None; }
  let block_ts = db.read_from_block_timestamps(number).unwrap();
  if block_ts.is_none() { return None; }
  let block_gas_used = db.read_from_block_gas_used(number).unwrap();
  if block_gas_used.is_none() { return None; }
  let block_mine_tm = db.read_from_block_mine_tm(number).unwrap();
  if block_mine_tm.is_none() { return None; }
  Some(BlockRes {
    number,
    timestamp: block_ts.unwrap(),
    gas_used: block_gas_used.unwrap(),
    hash: block_hash.unwrap(),
    mine_tm: block_mine_tm.unwrap(),
  })
}

fn get_latest_block_height(db: &DB) -> U256 {
  println!("Getting latest block height");
  let last_block_info = db.get_latest_block_hash().unwrap();
  if last_block_info.is_none() { return U256::ZERO; }
  db.get_latest_block_hash().unwrap().unwrap().0
}

fn mine_block(db: &DB, block_cnt: u64, timestamp: U256, hash: B256) {
  let mut number = get_latest_block_height(db) + U256::from(1);
  println!("Mining blocks from {} to {}", number, number + U256::from(block_cnt) - U256::from(1));
  let mut rwtxn = db.get_write_txn().unwrap();
  for _ in 0..block_cnt {
    db.set_block_hash_with_txn(number, hash, &mut rwtxn).unwrap();
    db.set_block_timestamps_with_txn(number, timestamp, &mut rwtxn).unwrap();
    db.set_block_gas_used_with_txn(number, U256::ZERO, &mut rwtxn).unwrap();
    db.set_block_mine_tm_with_txn(number, U256::ZERO, &mut rwtxn).unwrap();
    number += U256::from(1);
  }
  rwtxn.commit().unwrap();
}

fn add_tx_to_block(db_mutex: &Mutex<DB>, timestamp: U256, tx_info: &TxInfo) -> (ExecutionResult, u64, String) {
  let mut db_mutex_guard = db_mutex.lock().unwrap();
  let mut db = core::mem::take(&mut *db_mutex_guard);

  let number = get_latest_block_height(&db) + U256::from(1);
  let block_info: BlockEnv = BlockEnv {
    number,
    coinbase: "0x0000000000000000000000000000000000003Ca6".parse().unwrap(),
    timestamp,
    ..Default::default()
  };

  let nonce = db.basic(tx_info.from).unwrap().map(|x| x.nonce).unwrap_or(0);
  let txhash = get_tx_hash(&tx_info, &nonce);
  let mut evm = get_evm(&block_info, db);
  evm = modify_evm_with_tx_env(evm,
    tx_info.from, 
    tx_info.to.map(|x| TransactTo::Call(x)).unwrap_or(TransactTo::Create(CreateScheme::Create)), 
    tx_info.data.clone());
  
  let output = evm.transact().unwrap();
  evm.context.evm.db.commit(output.state, &txhash, &number);
  *db_mutex_guard = evm.context.evm.db;

  (output.result, nonce, txhash)
}

fn finalise_block(db: &DB, timestamp: U256, hash: B256, total_gas_used: u64, block_tx_cnt: u64) -> () {
  let start_time = std::time::Instant::now();

  let number = get_latest_block_height(&db) + U256::from(1);
  println!("Finalising block {}, tx cnt: {}", number, block_tx_cnt);

  let mut rwtxn = db.get_write_txn().unwrap();
  db.set_block_timestamps_with_txn(number, timestamp, &mut rwtxn).unwrap();
  db.set_block_gas_used_with_txn(number, U256::from(total_gas_used), &mut rwtxn).unwrap();
  let total_time_took = U256::from(start_time.elapsed().as_nanos());
  db.set_block_mine_tm_with_txn(number, total_time_took, &mut rwtxn).unwrap();
  db.set_block_hash_with_txn(number, hash, &mut rwtxn).unwrap();
  rwtxn.commit().unwrap();
}

fn call_contract(db_mutex: &Mutex<DB>, tx_info: &TxInfo) -> (ExecutionResult, u64, String) {
  let mut db_mutex_guard = db_mutex.lock().unwrap();
  let mut db = core::mem::take(&mut *db_mutex_guard);

  let number = get_latest_block_height(&db) + U256::from(1);
  let timestamp = U256::from(std::time::UNIX_EPOCH.elapsed().unwrap().as_secs());
  let block_info: BlockEnv = BlockEnv {
    number,
    coinbase: "0x0000000000000000000000000000000000003Ca6".parse().unwrap(),
    timestamp,
    ..Default::default()
  };

  let nonce = db.basic(tx_info.from).unwrap().map(|x| x.nonce).unwrap_or(0);
  let txhash = get_tx_hash(&tx_info, &nonce);
  let mut evm = get_evm(&block_info, db);
  evm = modify_evm_with_tx_env(evm,
    tx_info.from, 
    tx_info.to.map(|x| TransactTo::Call(x)).unwrap_or(TransactTo::Create(CreateScheme::Create)), 
    tx_info.data.clone());
  
  let output = evm.transact().unwrap().result;
  *db_mutex_guard = evm.context.evm.db;

  (output, nonce, txhash)
}  
