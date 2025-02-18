use std::sync::Mutex;
use std::time::Instant;

use rust_embed::Embed;

use db::{
    types::{
        AddressED, BlockResponseED, Decode, LogED, LogResponseED, TxED, TxReceiptED, B2048ED,
        B256ED,
    },
    DB,
};
use revm::primitives::{
    alloy_primitives::logs_bloom, Address, BlockEnv, Bytes, ExecutionResult, TransactTo, B256, U256,
};
use revm::Database;

use crate::types::{get_result_reason, get_result_type, get_tx_hash, TxInfo};
use crate::{
    evm::{get_evm, modify_evm_with_tx_env},
    types::get_contract_address,
};

#[derive(Embed)]
#[folder = "contracts/"]
struct ContractAssets;

pub struct ServerInstance {
    pub db_mutex: Mutex<DB>,
    pub waiting_tx_cnt_mutex: Mutex<u64>,
    pub last_ts_mutex: Mutex<u64>,
    pub last_block_hash_mutex: Mutex<B256>,
    pub last_block_gas_used_mutex: Mutex<u64>,
    pub last_block_log_index_mutex: Mutex<u64>,
}

fn load_contract(file_name: &str) -> TxInfo {
    let file_content = ContractAssets::get(file_name);
    let file_content = file_content.unwrap();
    let data = String::from_utf8(file_content.data.to_vec()).unwrap();

    serde_json::from_str(&data).unwrap()
}

fn deploy_brc20_contract(instance: &ServerInstance) {
    let result =
        instance.add_tx_to_block(0, &load_contract("BRC20_Controller.json"), 0, B256::ZERO);
    assert!(result.is_ok());

    println!(
        "BRC20_Controller contract address: {:?}",
        result.unwrap().contract_address.unwrap().0.to_string()
    );

    instance.finalise_block(0, B256::ZERO, 1, None).unwrap();
}

impl ServerInstance {
    pub fn new(db: DB) -> Self {
        #[cfg(debug_assertions)]
        println!("Creating new server instance");

        let instance = ServerInstance {
            db_mutex: Mutex::new(db),
            waiting_tx_cnt_mutex: Mutex::new(0),
            last_ts_mutex: Mutex::new(0),
            last_block_hash_mutex: Mutex::new(B256::ZERO),
            last_block_gas_used_mutex: Mutex::new(0),
            last_block_log_index_mutex: Mutex::new(0),
        };

        if instance.get_latest_block_height() == 0 {
            deploy_brc20_contract(&instance);
        }

        instance
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
            "Mining blocks from 0x{:x} ({}) to 0x{:x} ({})",
            number,
            number,
            number + block_cnt - 1,
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
    ) -> Result<TxReceiptED, &'static str> {
        #[cfg(debug_assertions)]
        {
            let block_number = self.get_latest_block_height() + 1;
            println!("Adding tx {:?} to block {:?}", tx_idx, block_number);
        }

        let mut waiting_tx_cnt = self.waiting_tx_cnt_mutex.lock().unwrap();
        let mut last_ts = self.last_ts_mutex.lock().unwrap();
        let mut last_block_hash = self.last_block_hash_mutex.lock().unwrap();
        let mut last_block_gas_used = self.last_block_gas_used_mutex.lock().unwrap();
        let mut last_block_log_index = self.last_block_log_index_mutex.lock().unwrap();

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
            *last_block_log_index = 0;
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
                "Running EVM for tx 0x{:x} ({}) in block 0x{:x} ({}) with hash {:?}",
                tx_idx, tx_idx, number, number, hash
            );

            let mut db = self.db_mutex.lock().unwrap();
            let db_moved = core::mem::take(&mut *db);
            let mut evm = get_evm(block_info, db_moved, None);
            #[cfg(debug_assertions)]
            println!(
                "Adding tx 0x{:x} ({}) from: {:?} to: {:?} with data: {:?}",
                tx_idx, tx_idx, tx_info.from, tx_info.to, tx_info.data
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
            "Tx 0x{:x} ({}) added to block 0x{:x} ({}) with gas used 0x{:x} ({})",
            tx_idx,
            tx_idx,
            number,
            number,
            output.gas_used(),
            output.gas_used()
        );
        *last_block_gas_used += output.gas_used();

        let mut db = self.db_mutex.lock().unwrap();
        db.set_tx_receipt(
            &get_result_type(&output),
            &get_result_reason(&output),
            hash,
            number,
            get_contract_address(&output),
            tx_info.from,
            tx_info.to,
            &tx_info.data,
            txhash,
            tx_idx,
            &output.clone(),
            *last_block_gas_used,
            nonce,
            *last_block_log_index,
        )
        .unwrap();

        *last_block_log_index += output.logs().len() as u64;

        Ok(db.get_tx_receipt(txhash).unwrap().unwrap())
    }

    pub fn get_transaction_count(
        &self,
        account: Address,
        block_number: u64,
    ) -> Result<u64, &'static str> {
        #[cfg(debug_assertions)]
        println!(
            "Getting transaction count for account {:?} in block 0x{:x} ({})",
            account, block_number, block_number
        );

        let mut db = self.db_mutex.lock().unwrap();
        let tx_count = db.get_tx_count(Some(account), block_number).unwrap();
        Ok(tx_count)
    }

    pub fn get_block_transaction_count_by_number(
        &self,
        block_number: u64,
    ) -> Result<u64, &'static str> {
        #[cfg(debug_assertions)]
        println!(
            "Getting block tx count for block 0x{:x} ({})",
            block_number, block_number
        );

        let mut db = self.db_mutex.lock().unwrap();
        let tx_count = db.get_tx_count(None, block_number).unwrap();
        Ok(tx_count)
    }

    pub fn get_block_transaction_count_by_hash(
        &self,
        block_hash: B256,
    ) -> Result<u64, &'static str> {
        #[cfg(debug_assertions)]
        println!("Getting block tx count for block hash {:?}", block_hash);

        let mut db = self.db_mutex.lock().unwrap();
        let block_number = db.get_block_number(block_hash).unwrap().unwrap().to_u64();
        let tx_count = db.get_tx_count(None, block_number).unwrap();
        Ok(tx_count)
    }

    pub fn get_transaction_by_block_hash_and_index(
        &self,
        block_hash: B256,
        tx_idx: u64,
    ) -> Option<TxED> {
        #[cfg(debug_assertions)]
        println!(
            "Getting tx by block hash {:?} and index 0x{:x} ({})",
            block_hash, tx_idx, tx_idx
        );

        let mut db = self.db_mutex.lock().unwrap();
        let tx_hash = db
            .get_tx_hash_by_block_hash_and_index(block_hash, tx_idx)
            .unwrap();
        tx_hash.map(|x| db.get_tx_by_hash(x.0).unwrap()).unwrap()
    }

    pub fn get_transaction_by_block_number_and_index(
        &self,
        block_number: u64,
        tx_idx: u64,
    ) -> Option<TxED> {
        #[cfg(debug_assertions)]
        println!(
            "Getting tx by block number 0x{:x} ({}) and index 0x{:x} ({})",
            block_number, block_number, tx_idx, tx_idx
        );

        let mut db = self.db_mutex.lock().unwrap();
        let tx_hash = db
            .get_tx_hash_by_block_number_and_index(block_number, tx_idx)
            .unwrap();
        tx_hash.map(|x| db.get_tx_by_hash(x.0).unwrap()).unwrap()
    }

    pub fn get_transaction_by_hash(&self, tx_hash: B256) -> Option<TxED> {
        #[cfg(debug_assertions)]
        println!("Getting tx by hash {:?}", tx_hash);

        let mut db = self.db_mutex.lock().unwrap();
        db.get_tx_by_hash(tx_hash).unwrap()
    }

    pub fn get_transaction_receipt(&self, tx_hash: B256) -> Option<TxReceiptED> {
        #[cfg(debug_assertions)]
        println!("Getting tx receipt for {:?}", tx_hash);

        let mut db = self.db_mutex.lock().unwrap();
        db.get_tx_receipt(tx_hash).unwrap()
    }

    pub fn get_logs(
        &self,
        block_number_from: Option<u64>,
        block_number_to: Option<u64>,
        address: Option<Address>,
        topics: Option<Vec<B256>>,
    ) -> Vec<LogResponseED> {
        #[cfg(debug_assertions)]
        println!("Getting logs");

        let mut db = self.db_mutex.lock().unwrap();
        db.get_logs(
            block_number_from,
            block_number_to,
            address,
            topics.unwrap_or(Vec::new()),
        )
        .unwrap()
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
        let mut last_block_log_index = self.last_block_log_index_mutex.lock().unwrap();

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
            "Finalising block 0x{:x} ({}), tx cnt: 0x{:x} ({})",
            block_number, block_number, block_tx_cnt, block_tx_cnt
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
        *last_block_log_index = 0;

        Ok(())
    }

    pub fn finalise_block_with_txes(
        &self,
        timestamp: u64,
        block_hash: B256,
        txes: Vec<TxInfo>,
    ) -> Result<Vec<TxReceiptED>, &'static str> {
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

        let mut tx_receipts: Vec<TxReceiptED> = Vec::new();
        let mut tx_idx = 0;
        for tx in txes {
            let result = self.add_tx_to_block(timestamp, &tx, tx_idx, block_hash);
            tx_idx += 1;

            if result.is_err() {
                return Err(result.unwrap_err());
            } else {
                tx_receipts.push(result.unwrap());
            }
        }

        let result = self.finalise_block(timestamp, block_hash, tx_len as u64, Some(start_time));

        if result.is_err() {
            return Err(result.unwrap_err());
        }
        Ok(tx_receipts)
    }

    pub fn call_contract(&self, tx_info: &TxInfo) -> Result<TxReceiptED, &'static str> {
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
            let mut evm = get_evm(block_info, db_moved, Some(U256::from(1_000_000)));
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

        Ok(TxReceiptED {
            status: output.as_ref().unwrap().is_success() as u8,
            transaction_result: get_result_type(output.as_ref().unwrap()),
            reason: get_result_reason(output.as_ref().unwrap()),
            logs: LogED {
                logs: output.as_ref().unwrap().logs().to_vec(),
                log_index: 0,
            },
            gas_used: output.as_ref().unwrap().gas_used(),
            from: AddressED(tx_info.from),
            to: tx_info.to.map(AddressED),
            contract_address: get_contract_address(output.as_ref().unwrap()).map(AddressED),
            logs_bloom: B2048ED::decode(logs_bloom(output.as_ref().unwrap().logs()).to_vec())
                .unwrap(),
            hash: B256ED::from_b256(txhash),
            block_number: number,
            transaction_hash: B256ED::from_b256(txhash),
            transaction_index: 0,
            cumulative_gas_used: output.as_ref().unwrap().gas_used(),
            nonce,
        })
    }

    pub fn get_storage_at(&self, contract: Address, location: U256) -> U256 {
        #[cfg(debug_assertions)]
        println!(
            "Getting storage at {:?} for contract {:?}",
            location, contract
        );

        let mut db = self.db_mutex.lock().unwrap();
        let storage = db.storage(contract, location);

        storage.unwrap_or(U256::ZERO)
    }

    pub fn get_block_by_number(&self, block_number: u64) -> Option<BlockResponseED> {
        #[cfg(debug_assertions)]
        println!(
            "Getting block by number 0x{:x} ({})",
            block_number, block_number
        );

        let mut db = self.db_mutex.lock().unwrap();
        db.get_block(block_number).unwrap()
    }

    pub fn get_block_by_hash(&self, block_hash: B256) -> Option<BlockResponseED> {
        #[cfg(debug_assertions)]
        println!("Getting block by hash {:?}", block_hash);

        let mut db = self.db_mutex.lock().unwrap();
        let block_number = db.get_block_number(block_hash).unwrap();

        #[cfg(debug_assertions)]
        println!("Got block {:?} hash {:?}", block_number, block_hash);

        db.get_block(block_number.unwrap().to_u64()).unwrap()
    }

    pub fn get_contract_bytecode(&self, addr: Address) -> Option<Bytes> {
        #[cfg(debug_assertions)]
        println!("Getting contract bytecode for {:?}", addr);

        let mut db = self.db_mutex.lock().unwrap();
        let acct = db.basic(addr).unwrap().unwrap();
        let bytecode = db.get_code(acct.code_hash).unwrap().unwrap();

        Some(bytecode.0.bytes())
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
        println!(
            "Reorg to block 0x{:x} ({})",
            latest_valid_block_number, latest_valid_block_number
        );

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
