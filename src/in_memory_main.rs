use revm::{Database, Evm};
// use revm::InMemoryDB;
use revm::primitives::env::{ Env, BlobExcessGasAndPrice, TransactTo, BlockEnv };
use revm::primitives::specification::SpecId;
use revm::primitives::ruint::aliases::U256;
use revm::primitives::{ keccak256, Address, CreateScheme, ExecutionResult, HaltReason, OutOfGasError, Output, SuccessReason, B256 };
use revm::primitives::alloy_primitives::Bytes;
use revm::precompile::{ Precompile, PrecompileWithAddress, StandardPrecompileFn, PrecompileResult, Precompiles, PrecompileSpecId, Error as PrecompileError };

use rouille::Response;
use rouille::try_or_400;

use serde::Serialize;
use serde_json::Value;

use std::sync::{ Mutex, Arc };

use db::DB;

// type DBtoUse = DB;
type DBtoUse = revm::db::InMemoryDB;

#[derive(Serialize)]
struct BlockResJSON {
  number: String,
  timestamp: String,
  #[serde(rename = "gasUsed")]
  gas_used: String,
  #[serde(rename = "mineTm")]
  mine_tm: String,
  hash: String,
}

#[derive(Serialize)]
struct SerializableExecutionResult {
  txhash: String,
  nonce: u64,
  #[serde(rename = "txResult")]
  r#type: String,
  #[serde(rename = "endReason")]
  reason: String,
  #[serde(rename = "gasUsed")]
  gas_used: String,
  #[serde(rename = "gasRefunded")]
  gas_refunded: String,
  logs: Vec<SerializeableLog>,
  #[serde(rename = "callOutput")]
  call_output: Option<String>,
  #[serde(rename = "contractAddress")]
  contract_address: Option<String>,
}
#[derive(Serialize)]
struct SerializeableLog {
  address: String,
  topics: Vec<String>,
  data: String,
}

fn main() {
  // let db_outer = DBtoUse::new().unwrap(); // NOTE
  let db_outer = DBtoUse::default();
  let db_arc_mutex_outer = Arc::new(Mutex::new(db_outer));

  println!("Starting server!");

  rouille::start_server("127.0.0.1:18545", move |request| {
    let db_arc_mutex = db_arc_mutex_outer.clone();

    if request.method() == "POST" {
      let json: Value = try_or_400!(rouille::input::json_input(request));
      let method = json.get("method").unwrap().as_str().unwrap();
      let params = json.get("params").unwrap().as_object().unwrap().clone();
      if method == "custom_blockNumber" {
        let db = db_arc_mutex.lock().unwrap();
        Response::json(&get_latest_block_height(&db).to_string())
      } else if method == "custom_mine" {
        let db = db_arc_mutex.lock().unwrap();
        let block_cnt = params.get("block_cnt").unwrap().as_u64().unwrap();
        let timestamp = params.get("timestamp").unwrap().as_u64().unwrap();
        let hash = B256::ZERO;
        mine_block(&db, block_cnt, U256::from(timestamp), hash);
        Response::json::<Value>(&Value::Null)
      } else if method == "custom_getBlockByNumber" {
        let db = db_arc_mutex.lock().unwrap();
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
        let from = params.get("from").unwrap().as_str().unwrap().parse().unwrap();
        let to = params.get("to").unwrap().as_str().map(|x| x.parse().unwrap());
        let data = Bytes::from(hex::decode(params.get("data").unwrap().as_str().unwrap().to_string().split_at(2).1).unwrap());
        let txinfo = TxInfo { from, to, data };
        let (result, nonce) = call_contract(&db_arc_mutex, &txinfo);
        let txhash = get_tx_hash(&txinfo, &nonce);
        let serializeable_res = get_serializeable_execution_result(result, txhash, nonce);
        Response::json(&serializeable_res)
      } else if method == "get_contract_bytecode" {
        let addr = params.get("addr").unwrap().as_str().unwrap().parse().unwrap();
        let mut db = db_arc_mutex.lock().unwrap();
        let acct = db.basic(addr).unwrap().unwrap();
        let bytecode = db.code_by_hash(acct.code_hash).unwrap();
        Response::json(&hex::encode(bytecode.bytes()))
      } else if method == "custom_mineBlockWithTxes" {
        let timestamp = U256::from(params.get("timestamp").unwrap().as_u64().unwrap());
        let hash = B256::from_slice(hex::decode(params.get("hash").unwrap().as_str().unwrap().to_string().split_at(2).1).unwrap().as_slice());
        let txes = params.get("txes").unwrap().as_array().unwrap().iter().map(|tx| {
          let from = tx.get("from").unwrap().as_str().unwrap().to_string().parse().unwrap();
          let to = tx.get("to").unwrap().as_str().map(|x| x.to_string().parse().unwrap());
          let data = Bytes::from(hex::decode(tx.get("data").unwrap().as_str().unwrap().to_string().split_at(2).1).unwrap());
          TxInfo { from, to, data }
        }).collect();
        let (results, nonces) = mine_block_with_txes(&db_arc_mutex, timestamp, hash, &txes);
        let txhashes: Vec<String> = txes.iter().zip(nonces.iter()).map(|(tx, nonce)| get_tx_hash(tx, nonce)).collect();
        let serializeable_resses: Vec<SerializableExecutionResult> = results.iter().zip(txhashes.iter()).zip(nonces.iter()).map(|((res, txhash), nonce)| get_serializeable_execution_result(res.clone(), txhash.clone(), nonce.clone())).collect();
        Response::json(&serializeable_resses)
      } else {
        Response::json::<Value>(&Value::Null)
      }
    } else {
      Response::text("Send POST Req!!!")
    }
  });
}

fn get_tx_hash(txinfo: &TxInfo, nonce: &u64) -> String {
  let mut data = Vec::new();
  data.extend_from_slice(txinfo.from.as_slice());
  data.extend_from_slice(&nonce.to_be_bytes());
  if let Some(to) = txinfo.to {
    data.extend_from_slice(to.as_slice());
  } else {
    data.extend_from_slice(&[0; 20]);
  }
  data.extend_from_slice(&txinfo.data);
  hex::encode(keccak256(data).0)
}

fn get_serializeable_execution_result(old: ExecutionResult, txhash: String, nonce: u64) -> SerializableExecutionResult {
  match old {
    ExecutionResult::Success { gas_used, gas_refunded, logs, output, reason } => {
      SerializableExecutionResult {
        txhash: "0x".to_string() + txhash.as_str(),
        nonce,
        r#type: "Success".to_string(),
        reason: match reason {
          SuccessReason::Stop => "Stop".to_string(),
          SuccessReason::Return => "Return".to_string(),
          SuccessReason::SelfDestruct => "SelfDestruct".to_string(),
        },
        gas_used: gas_used.to_string(),
        gas_refunded: gas_refunded.to_string(),
        logs: logs.iter().map(|log| SerializeableLog {
          address: log.address.to_string(),
          topics: log.topics().iter().map(|topic| format!("{:?}", topic)).collect(),
          data: format!("{}", hex::encode(&log.data.data)),
        }).collect(),
        call_output: match output.clone() {
          Output::Call(data) => Some(format!("{}", hex::encode(data))),
          Output::Create(_, _) => None,
        },
        contract_address: match output.clone() {
          Output::Call(_) => None,
          Output::Create(_, addr) => addr.map(|x| x.to_string()),
        },
      }
    },
    ExecutionResult::Revert { gas_used, output } => {
      SerializableExecutionResult {
        txhash,
        nonce,
        r#type: "Revert".to_string(),
        reason: "".to_string(),
        gas_used: gas_used.to_string(),
        gas_refunded: "0".to_string(),
        logs: Vec::new(),
        call_output: Some(format!("{}", hex::encode(output))),
        contract_address: None,
      }
    },
    ExecutionResult::Halt { gas_used, reason } => {
      SerializableExecutionResult {
        txhash,
        nonce,
        r#type: "Halt".to_string(),
        reason: match reason {
          HaltReason::OutOfGas(OutOfGasError::Basic) => "OutOfGas::Basic".to_string(),
          HaltReason::OutOfGas(OutOfGasError::MemoryLimit) => "OutOfGas::MemoryLimit".to_string(),
          HaltReason::OutOfGas(OutOfGasError::Memory) => "OutOfGas::Memory".to_string(),
          HaltReason::OutOfGas(OutOfGasError::Precompile) => "OutOfGas::Precompile".to_string(),
          HaltReason::OutOfGas(OutOfGasError::InvalidOperand) => "OutOfGas::InvalidOperand".to_string(),
          HaltReason::OpcodeNotFound => "OpcodeNotFound".to_string(),
          HaltReason::InvalidFEOpcode => "InvalidFEOpcode".to_string(),
          HaltReason::InvalidJump => "InvalidJump".to_string(),
          HaltReason::NotActivated => "NotActivated".to_string(),
          HaltReason::StackUnderflow => "StackUnderflow".to_string(),
          HaltReason::StackOverflow => "StackOverflow".to_string(),
          HaltReason::OutOfOffset => "OutOfOffset".to_string(),
          HaltReason::CreateCollision => "CreateCollision".to_string(),
          HaltReason::PrecompileError => "PrecompileError".to_string(),
          HaltReason::NonceOverflow => "NonceOverflow".to_string(),
          HaltReason::CreateContractSizeLimit => "CreateContractSizeLimit".to_string(),
          HaltReason::CreateContractStartingWithEF => "CreateContractStartingWithEF".to_string(),
          HaltReason::CreateInitCodeSizeLimit => "CreateInitCodeSizeLimit".to_string(),
          HaltReason::OverflowPayment => "OverflowPayment".to_string(),
          HaltReason::StateChangeDuringStaticCall => "StateChangeDuringStaticCall".to_string(),
          HaltReason::CallNotAllowedInsideStatic => "CallNotAllowedInsideStatic".to_string(),
          HaltReason::OutOfFunds => "OutOfFunds".to_string(),
          HaltReason::CallTooDeep => "CallTooDeep".to_string(),
        },
        gas_used: gas_used.to_string(),
        gas_refunded: "0".to_string(),
        logs: Vec::new(),
        call_output: None,
        contract_address: None,
      }
    },
  }
}

fn get_evm(block_info: &BlockEnv, db: DBtoUse) -> Evm<(), DBtoUse> {
  let mut env = Env::default();
  env.cfg.chain_id = 331337;
  env.cfg.limit_contract_code_size = Some(usize::MAX);

  env.block.number = block_info.number;
  env.block.coinbase = block_info.coinbase;
  env.block.timestamp = block_info.timestamp;
  env.block.gas_limit = U256::MAX;
  env.block.basefee = U256::ZERO;
  env.block.difficulty = U256::ZERO;
  env.block.prevrandao = Some(B256::ZERO);
  env.block.blob_excess_gas_and_price = Some(BlobExcessGasAndPrice::new(0));

  env.tx.gas_limit = u64::MAX;
  env.tx.gas_price = U256::ZERO;
  env.tx.value = U256::ZERO;

  let mut evm = Evm::builder()
    .with_db(db)
    .with_env(Box::new(env))
    .with_spec_id(SpecId::BERLIN)
    .build();

  evm.handler.pre_execution.load_precompiles = Arc::new(load_precompiles);

  evm
}
fn load_precompiles() -> Precompiles {
  let mut precompiles = Precompiles::new(PrecompileSpecId::from_spec_id(SpecId::BERLIN)).clone();
  precompiles.extend([
    PrecompileWithAddress::from((
      "0x00000000000000000000000000000000000000ff".parse().unwrap(), 
      Precompile::Standard(identity_run as StandardPrecompileFn)
    )),
    PrecompileWithAddress::from((
      "0x00000000000000000000000000000000000000fe".parse().unwrap(), 
      Precompile::Standard(identity_run as StandardPrecompileFn)
    ))
  ]);
  precompiles
}
fn identity_run(input: &[u8], gas_limit: u64) -> PrecompileResult {
  println!("Running identity precompile");
  let gas_used = 100000;
  if gas_used > gas_limit {
    return Err(PrecompileError::OutOfGas);
  }
  Ok((gas_used, input.to_vec()))
}
fn modify_evm_with_tx_env(evm: Evm<(), DBtoUse> , caller: Address, transact_to: TransactTo, data: Bytes) -> Evm<(), DBtoUse> {
  evm.modify().modify_tx_env(|tx_env| {
    tx_env.caller = caller;
    tx_env.transact_to = transact_to;
    tx_env.data = data;
  }).build()
}

pub struct BlockRes {
  pub number: U256,
  pub timestamp: U256,
  pub gas_used: U256,
  pub mine_tm: U256,
  pub hash: B256,
}

fn get_block_by_number(db: &DBtoUse, number: U256) -> Option<BlockRes> {
  println!("Getting block by number {}", number);
  /* let block_hash = db.read_from_block_hashes(number).unwrap();
  if block_hash.is_none() { return None; }
  let block_ts = db.read_from_block_timestamps(number).unwrap();
  if block_ts.is_none() { return None; }
  let block_gas_used = db.read_from_block_gas_used(number).unwrap();
  if block_gas_used.is_none() { return None; }
  let block_mine_tm = db.read_from_block_mine_tm(number).unwrap();
  if block_mine_tm.is_none() { return None; } */ // NOTE
  let block_hash = Some(B256::ZERO);
  let block_ts = Some(U256::ZERO);
  let block_gas_used = Some(U256::ZERO);
  let block_mine_tm = Some(U256::ZERO);
  Some(BlockRes {
    number,
    timestamp: block_ts.unwrap(),
    gas_used: block_gas_used.unwrap(),
    hash: block_hash.unwrap(),
    mine_tm: block_mine_tm.unwrap(),
  })
}

static mut LATEST_BLOCK_HEIGHT: U256 = U256::ZERO;
fn get_latest_block_height(db: &DBtoUse) -> U256 {
  println!("Getting latest block height");
  /* let last_block_info = db.get_latest_block_hash().unwrap();
  if last_block_info.is_none() { return U256::ZERO; }
  db.get_latest_block_hash().unwrap().unwrap().0 */ // NOTE
  unsafe { LATEST_BLOCK_HEIGHT }
}

fn mine_block(db: &DBtoUse, block_cnt: u64, timestamp: U256, hash: B256) {
  let mut number = get_latest_block_height(db) + U256::from(1);
  println!("Mining blocks from {} to {}", number, number + U256::from(block_cnt) - U256::from(1));
  /* let mut rwtxn = db.get_write_txn().unwrap();
  for _ in 0..block_cnt {
    db.set_block_hash_with_txn(number, hash, &mut rwtxn).unwrap();
    db.set_block_timestamps_with_txn(number, timestamp, &mut rwtxn).unwrap();
    db.set_block_gas_used_with_txn(number, U256::ZERO, &mut rwtxn).unwrap();
    db.set_block_mine_tm_with_txn(number, U256::ZERO, &mut rwtxn).unwrap();
    number += U256::from(1);
  }
  rwtxn.commit().unwrap(); */ // NOTE
  unsafe { LATEST_BLOCK_HEIGHT += U256::from(block_cnt) };
}

pub struct TxInfo {
  pub from: Address,
  pub to: Option<Address>,
  pub data: Bytes,
}
fn mine_block_with_txes(db_arc_mutex: &Arc<Mutex<DBtoUse>>, timestamp: U256, hash: B256, txes: &Vec<TxInfo>) -> (Vec<ExecutionResult>, Vec<u64>) {
  let start_time = std::time::Instant::now();
  let mut db_mutex_guard = db_arc_mutex.lock().unwrap();
  let db = core::mem::take(&mut *db_mutex_guard);

  let number = get_latest_block_height(&db) + U256::from(1);
  println!("Mining block {} with txes, cnt: {}", number, txes.len());

  let block_info: BlockEnv = BlockEnv {
    number,
    coinbase: "0x0000000000000000000000000000000000003Ca6".parse().unwrap(),
    timestamp,
    ..Default::default()
  };
  let mut total_gas_used = U256::ZERO;
  let mut results = Vec::new();
  let mut nonces: Vec<u64> = Vec::new();
  let mut db_inner = db;

  let mut evm = get_evm(&block_info, db_inner);
  for tx in txes {
    let nonce = evm.context.evm.db.basic(tx.from).unwrap().map(|x| x.nonce).unwrap_or(0);
    nonces.push(nonce);

    evm = modify_evm_with_tx_env(evm,
      tx.from, 
      tx.to.map(|x| TransactTo::Call(x)).unwrap_or(TransactTo::Create(CreateScheme::Create)), 
      tx.data.clone());
  
    let output = evm.transact_commit().unwrap();
    let gas_used = output.gas_used();
    results.push(output);
  
    // println!("gas used: {}", gas_used);
  
    total_gas_used += U256::from(gas_used);
  }
  db_inner = evm.context.evm.db;

  /* let mut rwtxn = db_inner.get_write_txn().unwrap();
  db_inner.set_block_hash_with_txn(number, hash, &mut rwtxn).unwrap();
  db_inner.set_block_timestamps_with_txn(number, timestamp, &mut rwtxn).unwrap();
  db_inner.set_block_gas_used_with_txn(number, total_gas_used, &mut rwtxn).unwrap();
  let total_time_took = U256::from(start_time.elapsed().as_nanos());
  db_inner.set_block_mine_tm_with_txn(number, total_time_took, &mut rwtxn).unwrap();
  rwtxn.commit().unwrap(); */ // NOTE
  unsafe { LATEST_BLOCK_HEIGHT += U256::from(1) };

  *db_mutex_guard = db_inner;
  (results, nonces)
}

fn call_contract(db_arc_mutex: &Arc<Mutex<DBtoUse>>, tx: &TxInfo) -> (ExecutionResult, u64) {
  let mut db_mutex_guard = db_arc_mutex.lock().unwrap();
  let mut db = core::mem::take(&mut *db_mutex_guard);

  let number = get_latest_block_height(&db) + U256::from(1);
  // let timestamp = db.read_from_block_timestamps(number - U256::from(1)).unwrap().unwrap(); // TODO: maybe use current time?? // NOTE
  let timestamp = U256::ZERO;
  let block_info: BlockEnv = BlockEnv {
    number,
    coinbase: "0x0000000000000000000000000000000000003Ca6".parse().unwrap(),
    timestamp,
    ..Default::default()
  };

  let nonce = db.basic(tx.from).unwrap().map(|x| x.nonce).unwrap_or(0);
  let mut evm = get_evm(&block_info, db);
  evm = modify_evm_with_tx_env(evm,
    tx.from, 
    tx.to.map(|x| TransactTo::Call(x)).unwrap_or(TransactTo::Create(CreateScheme::Create)), 
    tx.data.clone());
  
  let output = evm.transact().unwrap().result;
  *db_mutex_guard = evm.context.evm.db;

  (output, nonce)
}
