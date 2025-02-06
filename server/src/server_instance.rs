use std::sync::Mutex;
use std::time::Instant;

use db::DB;
use revm::primitives::{Address, BlockEnv, Bytes, ExecutionResult, TransactTo, B256, U256};
use revm::Database;

use crate::evm::{get_evm, modify_evm_with_tx_env};
use crate::types::{
    get_serializable_execution_result, get_tx_hash, BlockRes, SerializableExecutionResult, TxInfo,
};

pub struct ServerInstance {
    pub db_mutex: Mutex<DB>,
    pub waiting_tx_cnt_mutex: Mutex<u64>,
    pub last_ts_mutex: Mutex<u64>,
    pub last_block_hash_mutex: Mutex<B256>,
    pub last_block_gas_used_mutex: Mutex<u64>,
}

impl ServerInstance {
    pub fn new(db: DB) -> Self {
        #[cfg(debug_assertions)]
        println!("Creating new server instance");

        ServerInstance {
            db_mutex: Mutex::new(db),
            waiting_tx_cnt_mutex: Mutex::new(0),
            last_ts_mutex: Mutex::new(0),
            last_block_hash_mutex: Mutex::new(B256::ZERO),
            last_block_gas_used_mutex: Mutex::new(0),
        }
    }

    pub fn get_latest_block_height(&self) -> u64 {
        let db = self.db_mutex.lock().unwrap();
        let last_block_info = db.get_latest_block_height();

        let block_height = last_block_info.unwrap_or(0);
        block_height
    }

    pub fn mine_block(
        &self,
        block_cnt: u64,
        timestamp: u64,
        hash: B256,
    ) -> Result<(), &'static str> {
        let waiting_tx_cnt = self.waiting_tx_cnt_mutex.lock().unwrap();
        if *waiting_tx_cnt != 0 {
            return Err("There are waiting txes committed to db, cannot mine empty block!");
        }

        let mut number = self.get_latest_block_height() + 1;

        let mut db = self.db_mutex.lock().unwrap();

        #[cfg(debug_assertions)]
        println!(
            "Mining blocks from {} to {}",
            number,
            number + block_cnt - 1
        );

        let mut number_clone = number.clone();

        for _ in 0..block_cnt {
            db.set_block_hash(number, hash).unwrap();
            db.set_block_timestamp(number, timestamp).unwrap();
            number += 1;
        }

        for _ in 0..block_cnt {
            db.set_gas_used(number_clone, 0).unwrap();
            db.set_mine_timestamp(number_clone, 0).unwrap();
            number_clone += 1;
        }

        Ok(())
    }

    pub fn add_tx_to_block(
        &self,
        timestamp: u64,
        tx_info: &TxInfo,
        tx_idx: u64,
        hash: B256,
    ) -> Result<SerializableExecutionResult, &'static str> {
        #[cfg(debug_assertions)]
        {
            let block_number = self.get_latest_block_height() + 1;
            println!("Adding tx {:?} to block {:?}", tx_idx, block_number);
        }

        let mut waiting_tx_cnt = self.waiting_tx_cnt_mutex.lock().unwrap();
        let mut last_ts = self.last_ts_mutex.lock().unwrap();
        let mut last_block_hash = self.last_block_hash_mutex.lock().unwrap();
        let mut last_block_gas_used = self.last_block_gas_used_mutex.lock().unwrap();

        if *waiting_tx_cnt != tx_idx {
            return Err("tx_idx is different from waiting tx cnt in block!!");
        }
        if *waiting_tx_cnt != 0 {
            if timestamp != *last_ts {
                return Err("timestamp is different from other txes in block!!");
            }

            if hash != *last_block_hash {
                return Err("block hash is different from other txes in block!!");
            }
        } else {
            *last_ts = timestamp;
            *last_block_hash = hash;
            *last_block_gas_used = 0;
        }

        let number = self.get_latest_block_height() + 1;

        let block_info: BlockEnv = BlockEnv {
            number: U256::from_limbs([0, 0, 0, number]),
            coinbase: "0x0000000000000000000000000000000000003Ca6"
                .parse()
                .unwrap(),
            timestamp: U256::from(timestamp),
            ..Default::default()
        };

        let output: Option<ExecutionResult>;
        let nonce = self.get_nonce(tx_info.from);
        let txhash = get_tx_hash(&tx_info, &nonce);

        {
            #[cfg(debug_assertions)]
            println!(
                "Running EVM for tx {:?} in block {:?} with hash {:?}",
                tx_idx, number, hash
            );

            let mut db = self.db_mutex.lock().unwrap();
            let db_moved = core::mem::take(&mut *db);
            let mut evm = get_evm(block_info, db_moved);
            #[cfg(debug_assertions)]
            println!(
                "Adding tx {:?} from: {:?} to: {:?} with data: {:?}",
                tx_idx, tx_info.from, tx_info.to, tx_info.data
            );
            evm = modify_evm_with_tx_env(
                evm,
                tx_info.from,
                tx_info
                    .to
                    .map(|x| TransactTo::Call(x))
                    .unwrap_or(TransactTo::Create),
                tx_info.data.clone(),
            );

            output = Some(evm.transact_commit().unwrap());
            core::mem::swap(&mut *db, &mut evm.context.evm.db);
        }

        let output = output.unwrap();

        *waiting_tx_cnt += 1;

        #[cfg(debug_assertions)]
        println!(
            "Tx {:?} added to block {:?} with gas used {:?}",
            tx_idx,
            number,
            output.gas_used()
        );
        *last_block_gas_used += output.gas_used();

        Ok(get_serializable_execution_result(output, txhash, nonce))
    }

    pub fn finalise_block(
        &self,
        timestamp: u64,
        block_hash: B256,
        block_tx_cnt: u64,
        mut start_time: Option<Instant>,
    ) -> Result<(), &'static str> {
        if start_time.is_none() {
            start_time = Some(Instant::now());
        }
        let mut waiting_tx_cnt = self.waiting_tx_cnt_mutex.lock().unwrap();
        let mut last_ts = self.last_ts_mutex.lock().unwrap();
        let mut last_block_hash = self.last_block_hash_mutex.lock().unwrap();
        let mut last_block_gas_used = self.last_block_gas_used_mutex.lock().unwrap();

        if *waiting_tx_cnt != 0 {
            if timestamp != *last_ts {
                return Err("timestamp is different from other txes in block!!");
            }

            if block_hash != *last_block_hash {
                return Err("block hash is different from other txes in block!!");
            }
        } else {
            *last_block_gas_used = 0; // not needed but just for sanity
        }
        if *waiting_tx_cnt != block_tx_cnt {
            return Err("block tx cnt is different from waiting tx cnt for block!!");
        }

        let block_number = self.get_latest_block_height() + 1;
        let mut db = self.db_mutex.lock().unwrap();

        #[cfg(debug_assertions)]
        println!(
            "Finalising block {}, tx cnt: {}",
            block_number, block_tx_cnt
        );

        db.set_block_hash(block_number, block_hash).unwrap();

        db.set_gas_used(block_number, *last_block_gas_used).unwrap();
        let total_time_took = start_time.unwrap().elapsed().as_nanos();
        db.set_mine_timestamp(block_number, total_time_took)
            .unwrap();
        db.set_block_timestamp(block_number, timestamp).unwrap();
        db.set_block_hash(block_number, block_hash).unwrap();

        *waiting_tx_cnt = 0;
        *last_ts = 0;
        *last_block_hash = B256::ZERO;
        *last_block_gas_used = 0;

        Ok(())
    }

    pub fn finalise_block_with_txes(
        &self,
        timestamp: u64,
        block_hash: B256,
        txes: Vec<TxInfo>,
    ) -> Result<Vec<SerializableExecutionResult>, &'static str> {
        #[cfg(debug_assertions)]
        println!("Finalising block with {:?} txes", txes.len());

        {
            let mut waiting_tx_cnt = self.waiting_tx_cnt_mutex.lock().unwrap();

            if *waiting_tx_cnt != 0 {
                return Err("there are waiting txes, either finalise block or clear caches!!");
            }

            let mut last_block_gas_used = self.last_block_gas_used_mutex.lock().unwrap();

            *waiting_tx_cnt = 0;
            *last_block_gas_used = 0;
        }

        let start_time = Instant::now();
        let tx_len = txes.len();

        let mut serializable_results: Vec<SerializableExecutionResult> = Vec::new();
        for tx in txes {
            let result = self.add_tx_to_block(timestamp, &tx, 0, block_hash);

            if result.is_err() {
                return Err(result.unwrap_err());
            } else {
                serializable_results.push(result.unwrap());
            }
        }

        let result = self.finalise_block(timestamp, block_hash, tx_len as u64, Some(start_time));

        if result.is_err() {
            return Err(result.unwrap_err());
        }
        Ok(serializable_results)
    }

    pub fn call_contract(
        &self,
        tx_info: &TxInfo,
    ) -> Result<SerializableExecutionResult, &'static str> {
        #[cfg(debug_assertions)]
        println!(
            "Calling contract from: {:?} to: {:?}",
            tx_info.from, tx_info.to
        );

        let waiting_tx_cnt = self.waiting_tx_cnt_mutex.lock().unwrap();
        if *waiting_tx_cnt != 0 {
            return Err("There are waiting txes committed to db, cannot mine empty block!");
        }
        let number = self.get_latest_block_height() + 1;

        let timestamp = U256::from(std::time::UNIX_EPOCH.elapsed().unwrap().as_secs());
        let block_info: BlockEnv = BlockEnv {
            number: U256::from_limbs([0, 0, 0, number]),
            coinbase: "0x0000000000000000000000000000000000003Ca6"
                .parse()
                .unwrap(),
            timestamp,
            ..Default::default()
        };

        let output: Option<ExecutionResult>;
        let nonce = self.get_nonce(tx_info.from);
        let txhash = get_tx_hash(&tx_info, &nonce);

        {
            let mut db = self.db_mutex.lock().unwrap();
            let db_moved = core::mem::take(&mut *db);
            let mut evm = get_evm(block_info, db_moved);
            evm = modify_evm_with_tx_env(
                evm,
                tx_info.from,
                tx_info
                    .to
                    .map(|x| TransactTo::Call(x))
                    .unwrap_or(TransactTo::Create),
                tx_info.data.clone(),
            );

            output = Some(evm.transact().unwrap().result);
            core::mem::swap(&mut *db, &mut evm.context.evm.db);
        }

        Ok(get_serializable_execution_result(
            output.unwrap(),
            txhash,
            nonce,
        ))
    }

    pub fn get_block_by_number(&self, block_number: u64) -> Option<BlockRes> {
        #[cfg(debug_assertions)]
        println!("Getting block by number {}", block_number);

        let mut db = self.db_mutex.lock().unwrap();
        let block_hash = db.get_block_hash(block_number).unwrap();
        if block_hash.is_none() {
            return None;
        }
        let block_ts = db.get_block_timestamp(block_number).unwrap();
        if block_ts.is_none() {
            return None;
        }
        let block_gas_used = db.get_gas_used(block_number).unwrap();
        if block_gas_used.is_none() {
            return None;
        }
        let block_mine_tm = db.get_mine_timestamp(block_number).unwrap();
        if block_mine_tm.is_none() {
            return None;
        }
        let block_res = BlockRes {
            number: block_number,
            timestamp: block_ts.unwrap().as_limbs()[0],
            gas_used: block_gas_used.unwrap().as_limbs()[0],
            hash: block_hash.unwrap(),
            mine_tm: block_mine_tm.unwrap(),
        };

        #[cfg(debug_assertions)]
        println!(
            "Got block {:?} with hash {:?}",
            block_number, block_res.hash
        );

        Some(block_res)
    }

    pub fn get_block_by_hash(&self, block_hash: B256) -> Option<BlockRes> {
        #[cfg(debug_assertions)]
        println!("Getting block by hash {:?}", block_hash);

        let mut db = self.db_mutex.lock().unwrap();
        let block_number = db.get_block_number(block_hash).unwrap();

        #[cfg(debug_assertions)]
        println!("Got block {:?} hash {:?}", block_number, block_hash);

        self.get_block_by_number(block_number.unwrap())
    }

    pub fn get_contract_bytecode(&self, addr: Address) -> Option<Bytes> {
        #[cfg(debug_assertions)]
        println!("Getting contract bytecode for {:?}", addr);

        let mut db = self.db_mutex.lock().unwrap();
        let acct = db.basic(addr).unwrap().unwrap();
        let bytecode = db.get_code(acct.code_hash).unwrap().unwrap();

        Some(bytecode.bytes())
    }

    pub fn clear_caches(&self) {
        #[cfg(debug_assertions)]
        println!("Clearing caches");

        let mut waiting_tx_cnt = self.waiting_tx_cnt_mutex.lock().unwrap();
        let mut last_ts = self.last_ts_mutex.lock().unwrap();
        let mut last_block_hash = self.last_block_hash_mutex.lock().unwrap();
        let mut last_block_gas_used = self.last_block_gas_used_mutex.lock().unwrap();
        let mut db = self.db_mutex.lock().unwrap();

        db.clear_caches();

        *waiting_tx_cnt = 0;
        *last_ts = 0;
        *last_block_hash = B256::ZERO;
        *last_block_gas_used = 0;
    }

    pub fn commit_to_db(&self) -> Result<(), &'static str> {
        #[cfg(debug_assertions)]
        println!("Committing to db");

        let waiting_tx_cnt = self.waiting_tx_cnt_mutex.lock().unwrap();
        if *waiting_tx_cnt != 0 {
            return Err("there are waiting txes, first finalise the block or clear the cache!!");
        }

        let mut db = self.db_mutex.lock().unwrap();
        db.commit_changes().unwrap();
        Ok(())
    }

    pub fn reorg(&self, latest_valid_block_number: u64) -> Result<(), &'static str> {
        #[cfg(debug_assertions)]
        println!("Reorging to block {}", latest_valid_block_number);

        let waiting_tx_cnt = self.waiting_tx_cnt_mutex.lock().unwrap();
        if *waiting_tx_cnt != 0 {
            return Err("there are waiting txes, first finalise the block or clear the cache!!");
        }

        let mut db = self.db_mutex.lock().unwrap();
        db.reorg(latest_valid_block_number).unwrap();
        Ok(())
    }

    fn get_nonce(&self, addr: Address) -> u64 {
        let mut db = self.db_mutex.lock().unwrap();
        db.basic(addr).unwrap().map_or(0, |x| x.nonce)
    }
}
