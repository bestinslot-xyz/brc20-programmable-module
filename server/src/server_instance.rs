use std::sync::Mutex;
use std::time::Instant;

use db::{
    types::{
        AddressED, BlockResponseED, Decode, LogED, LogResponseED, TxED, TxReceiptED, B2048ED,
        B256ED,
    },
    DB,
};
use revm::{
    primitives::{
        alloy_primitives::logs_bloom, Address, BlockEnv, Bytes, ExecutionResult, TransactTo, B256,
        U256,
    },
    Database,
};

use crate::brc20_controller::{load_brc20_deploy_tx, verify_brc20_contract_address};

use crate::evm::{
    get_contract_address, get_evm, get_result_reason, get_result_type, modify_evm_with_tx_env,
};

use crate::types::{get_tx_hash, TxInfo};

pub struct LastBlockInfo {
    pub waiting_tx_cnt: u64,
    pub last_ts: u64,
    pub last_block_hash: B256,
    pub last_block_gas_used: u64,
    pub last_block_log_index: u64,
    pub last_block_start_time: Option<Instant>,
}

impl LastBlockInfo {
    pub fn new() -> Self {
        LastBlockInfo {
            waiting_tx_cnt: 0,
            last_ts: 0,
            last_block_hash: B256::ZERO,
            last_block_gas_used: 0,
            last_block_log_index: 0,
            last_block_start_time: None,
        }
    }
}

pub struct ServerInstance {
    pub db_mutex: Mutex<DB>,
    pub last_block_info: Mutex<LastBlockInfo>,
}

impl ServerInstance {
    pub fn new(db: DB) -> Self {
        #[cfg(debug_assertions)]
        println!("Creating new server instance");

        let instance = ServerInstance {
            db_mutex: Mutex::new(db),
            last_block_info: Mutex::new(LastBlockInfo::new()),
        };

        instance
    }

    pub fn initialise(
        &self,
        genesis_hash: B256,
        genesis_timestamp: u64,
    ) -> Result<(), &'static str> {
        #[cfg(debug_assertions)]
        println!(
            "Initialising server instance with genesis hash {:?} and timestamp {}",
            genesis_hash, genesis_timestamp
        );

        let genesis = self.get_block_by_number(0);

        if genesis.is_some() {
            let genesis = genesis.unwrap();
            if genesis.hash.0 == genesis_hash {
                return Ok(());
            } else {
                return Err("Genesis block hash mismatch");
            }
        }

        // Deploy BRC20 Controller contract
        let result = self.add_tx_to_block(
            genesis_timestamp,
            &load_brc20_deploy_tx(),
            0,
            0,
            genesis_hash,
        )?;

        let brc20_controller_contract = result.contract_address.unwrap().0;
        verify_brc20_contract_address(&brc20_controller_contract.to_string());

        self.finalise_block(genesis_timestamp, 0, genesis_hash, 1)?;

        assert!(self.get_latest_block_height() == 0);
        Ok(())
    }

    pub fn get_latest_block_height(&self) -> u64 {
        let db = self.db_mutex.lock().unwrap();
        let latest_block_height = db.get_latest_block_height();

        let block_height = latest_block_height.unwrap_or(0);
        block_height
    }

    pub fn mine_block(
        &self,
        block_cnt: u64,
        timestamp: u64,
        hash: B256,
    ) -> Result<(), &'static str> {
        self.require_no_waiting_txes()?;

        let mut number = self.get_latest_block_height() + 1;

        #[cfg(debug_assertions)]
        println!(
            "Mining blocks from 0x{:x} ({}) to 0x{:x} ({})",
            number,
            number,
            number + block_cnt - 1,
            number + block_cnt - 1
        );

        for _ in 0..block_cnt {
            self.finalise_block(timestamp, number, hash, 0)?;
            number += 1;
        }

        Ok(())
    }

    pub fn add_tx_to_block(
        &self,
        timestamp: u64,
        tx_info: &TxInfo,
        tx_idx: u64,
        block_number: u64,
        block_hash: B256,
    ) -> Result<TxReceiptED, &'static str> {
        #[cfg(debug_assertions)]
        println!("Adding tx {:?} to block {:?}", tx_idx, block_number);

        let mut last_block_info = self.last_block_info.lock().unwrap();

        if last_block_info.waiting_tx_cnt != tx_idx {
            return Err("tx_idx is different from waiting tx cnt in block!!");
        }
        if last_block_info.waiting_tx_cnt != 0 {
            if timestamp != last_block_info.last_ts {
                return Err("timestamp is different from other txes in block!!");
            }

            if block_hash != last_block_info.last_block_hash {
                return Err("block hash is different from other txes in block!!");
            }
        } else {
            *last_block_info = LastBlockInfo {
                waiting_tx_cnt: 0,
                last_ts: timestamp,
                last_block_hash: block_hash,
                last_block_gas_used: 0,
                last_block_log_index: 0,
                last_block_start_time: Instant::now().into(),
            };
        }

        let block_info: BlockEnv = BlockEnv {
            number: U256::from_limbs([0, 0, 0, block_number]),
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
                tx_idx, tx_idx, block_number, block_number, block_hash
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

        last_block_info.waiting_tx_cnt += 1;

        #[cfg(debug_assertions)]
        println!(
            "Tx 0x{:x} ({}) added to block 0x{:x} ({}) with gas used 0x{:x} ({})",
            tx_idx,
            tx_idx,
            block_number,
            block_number,
            output.gas_used(),
            output.gas_used()
        );
        last_block_info.last_block_gas_used += output.gas_used();

        let mut db = self.db_mutex.lock().unwrap();
        db.set_tx_receipt(
            &get_result_type(&output),
            &get_result_reason(&output),
            output.output(),
            block_hash,
            block_number,
            get_contract_address(&output),
            tx_info.from,
            tx_info.to,
            &tx_info.data,
            txhash,
            tx_idx,
            &output.clone(),
            last_block_info.last_block_gas_used,
            nonce,
            last_block_info.last_block_log_index,
        )
        .unwrap();

        last_block_info.last_block_log_index += output.logs().len() as u64;

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
        let tx_count = db.get_tx_count(Some(account), block_number).unwrap_or(0);
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
        let tx_count = db.get_tx_count(None, block_number).unwrap_or(0);
        Ok(tx_count)
    }

    pub fn get_block_transaction_count_by_hash(
        &self,
        block_hash: B256,
    ) -> Result<u64, &'static str> {
        #[cfg(debug_assertions)]
        println!("Getting block tx count for block hash {:?}", block_hash);

        let mut db = self.db_mutex.lock().unwrap();
        let block_number = db.get_block_number(block_hash);
        if block_number.is_err() {
            return Err("block hash not found");
        }
        let block_number = block_number.unwrap().map(|x| x.to_u64()).unwrap_or(0);
        let tx_count = db.get_tx_count(None, block_number).unwrap_or(0);
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
        tx_hash
            .map(|x| db.get_tx_by_hash(x.0).unwrap_or(None))
            .unwrap()
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
        tx_hash
            .map(|x| db.get_tx_by_hash(x.0).unwrap_or(None))
            .unwrap()
    }

    pub fn get_transaction_by_hash(&self, tx_hash: B256) -> Option<TxED> {
        #[cfg(debug_assertions)]
        println!("Getting tx by hash {:?}", tx_hash);

        let mut db = self.db_mutex.lock().unwrap();
        db.get_tx_by_hash(tx_hash).unwrap_or(None)
    }

    pub fn get_transaction_receipt(&self, tx_hash: B256) -> Option<TxReceiptED> {
        #[cfg(debug_assertions)]
        println!("Getting tx receipt for {:?}", tx_hash);

        let mut db = self.db_mutex.lock().unwrap();
        db.get_tx_receipt(tx_hash).unwrap_or(None)
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
        .unwrap_or(Vec::new())
    }

    pub fn finalise_block(
        &self,
        timestamp: u64,
        block_number: u64,
        block_hash: B256,
        block_tx_cnt: u64,
    ) -> Result<(), &'static str> {
        let mut last_block_info = self.last_block_info.lock().unwrap();

        if last_block_info.last_block_start_time.is_none() {
            last_block_info.last_block_start_time = Some(Instant::now());
        }

        if last_block_info.waiting_tx_cnt != 0 {
            if timestamp != last_block_info.last_ts {
                return Err("timestamp is different from other txes in block!!");
            }

            if block_hash != last_block_info.last_block_hash {
                return Err("block hash is different from other txes in block!!");
            }
        }
        if last_block_info.waiting_tx_cnt != block_tx_cnt {
            return Err("block tx cnt is different from waiting tx cnt for block!!");
        }

        let mut db = self.db_mutex.lock().unwrap();

        #[cfg(debug_assertions)]
        println!(
            "Finalising block 0x{:x} ({}), tx cnt: 0x{:x} ({})",
            block_number, block_number, block_tx_cnt, block_tx_cnt
        );

        db.set_block_hash(block_number, block_hash).unwrap();

        db.set_gas_used(block_number, last_block_info.last_block_gas_used)
            .unwrap();
        let total_time_took = last_block_info
            .last_block_start_time
            .unwrap()
            .elapsed()
            .as_nanos();
        db.set_mine_timestamp(block_number, total_time_took)
            .unwrap();
        db.set_block_timestamp(block_number, timestamp).unwrap();
        db.set_block_hash(block_number, block_hash).unwrap();

        *last_block_info = LastBlockInfo::new();

        Ok(())
    }

    pub fn call_contract(&self, tx_info: &TxInfo) -> Result<TxReceiptED, &'static str> {
        #[cfg(debug_assertions)]
        println!(
            "Calling contract from: {:?} to: {:?}",
            tx_info.from, tx_info.to
        );
        self.require_no_waiting_txes()?;

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

        let output;
        let nonce = self.get_nonce(tx_info.from);
        let txhash = get_tx_hash(&tx_info, &nonce);

        {
            let mut db = self.db_mutex.lock().unwrap();
            let db_moved = core::mem::take(&mut *db);
            let mut evm = get_evm(block_info, db_moved, None);
            evm = modify_evm_with_tx_env(
                evm,
                tx_info.from,
                tx_info
                    .to
                    .map(|x| TransactTo::Call(x))
                    .unwrap_or(TransactTo::Create),
                tx_info.data.clone(),
            );

            output = evm.transact().map(|x| x.result);
            core::mem::swap(&mut *db, &mut evm.context.evm.db);
        }

        if output.is_err() {
            return Err("Error while calling contract");
        }

        Ok(TxReceiptED {
            status: output.as_ref().unwrap().is_success() as u8,
            transaction_result: get_result_type(output.as_ref().unwrap()),
            reason: get_result_reason(output.as_ref().unwrap()),
            result_bytes: output.as_ref().unwrap().output().cloned(),
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

        if block_number.is_none() {
            return None;
        }

        #[cfg(debug_assertions)]
        println!("Got block {:?} hash {:?}", block_number, block_hash);

        db.get_block(block_number.unwrap().to_u64()).unwrap()
    }

    pub fn get_contract_bytecode(&self, addr: Address) -> Option<Bytes> {
        #[cfg(debug_assertions)]
        println!("Getting contract bytecode for {:?}", addr);

        let mut db = self.db_mutex.lock().unwrap();
        let acct = db.basic(addr).unwrap();
        if acct.is_none() {
            return None;
        }
        let bytecode = db.get_code(acct.unwrap().code_hash).unwrap();

        bytecode.map(|x| x.0.bytes())
    }

    pub fn clear_caches(&self) {
        #[cfg(debug_assertions)]
        println!("Clearing caches");

        let mut db = self.db_mutex.lock().unwrap();
        let mut last_block_info = self.last_block_info.lock().unwrap();

        db.clear_caches();
        *last_block_info = LastBlockInfo::new();
    }

    pub fn commit_to_db(&self) -> Result<(), &'static str> {
        #[cfg(debug_assertions)]
        println!("Committing to db");
        self.require_no_waiting_txes()?;

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

        self.require_no_waiting_txes()?;

        let mut db = self.db_mutex.lock().unwrap();
        db.reorg(latest_valid_block_number).unwrap();
        Ok(())
    }

    fn require_no_waiting_txes(&self) -> Result<(), &'static str> {
        let last_block_info = self.last_block_info.lock().unwrap();
        if last_block_info.waiting_tx_cnt != 0 {
            return Err("there are waiting txes, either finalise block or clear caches!!");
        }
        Ok(())
    }

    fn get_nonce(&self, addr: Address) -> u64 {
        let mut db = self.db_mutex.lock().unwrap();
        db.basic(addr).unwrap().map_or(0, |x| x.nonce)
    }
}
