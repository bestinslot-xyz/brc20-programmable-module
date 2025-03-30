use std::sync::Mutex;
use std::time::Instant;

use revm::context::result::ExecutionResult;
use revm::context::{BlockEnv, ContextTr, TransactTo};
use revm::handler::{EvmTr, ExecuteCommitEvm};
use revm::primitives::alloy_primitives::logs_bloom;
use revm::primitives::{Address, Bytes, B256, U256};
use revm::{Database, ExecuteEvm};

use crate::brc20_controller::{load_brc20_deploy_tx, verify_brc20_contract_address};
use crate::db::types::{
    AddressED, BlockResponseED, Decode, LogED, LogResponseED, TxED, TxReceiptED, B2048ED, B256ED,
};
use crate::db::{DB, MAX_HISTORY_SIZE};
use crate::evm::{
    get_brc20_balance, get_contract_address, get_evm, get_gas_limit, get_result_reason,
    get_result_type,
};
use crate::server::types::{get_tx_hash, TxInfo};

pub struct LastBlockInfo {
    pub waiting_tx_count: u64,
    pub last_ts: u64,
    pub last_block_hash: B256,
    pub last_block_gas_used: u64,
    pub last_block_log_index: u64,
    pub last_block_start_time: Option<Instant>,
}

impl LastBlockInfo {
    pub fn new() -> Self {
        LastBlockInfo {
            waiting_tx_count: 0,
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
        genesis_height: u64,
    ) -> Result<(), &'static str> {
        #[cfg(debug_assertions)]
        println!(
            "Initialising server instance with genesis hash {:?} and timestamp {} at height {}",
            genesis_hash, genesis_timestamp, genesis_height
        );

        let genesis = self.get_block_by_number(genesis_height, false);

        println!("Genesis block: {:?}", genesis);
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
            genesis_height,
            genesis_hash,
            None,
            Some(u64::MAX),
        )?;

        let brc20_controller_contract = result.contract_address.unwrap().0;
        verify_brc20_contract_address(&brc20_controller_contract.to_string());

        self.finalise_block(genesis_timestamp, genesis_height, genesis_hash, 1)?;

        // Check status of BRC20 Balance Server before proceeding
        get_brc20_balance(&Bytes::from([10]), &Bytes::from([10]))
            .map_err(|_| "BRC20 Balance Server is down. This error can be ignored in tests that doesn't involve the BRC20 indexer.")?;

        Ok(())
    }

    pub fn get_next_block_height(&self) -> u64 {
        let mut db = self.db_mutex.lock().unwrap();
        let latest_block_height = db.get_latest_block_height();

        let block_height = latest_block_height.unwrap_or(0);
        if block_height == 0 {
            // Check if block 0 exists, then next block would be genesis
            let block_hash = db.get_block_hash(0);
            if block_hash.is_err() || block_hash.unwrap().is_none() {
                return 0;
            }
            return 1;
        }

        block_height + 1
    }

    pub fn get_latest_block_height(&self) -> u64 {
        let db = self.db_mutex.lock().unwrap();
        let latest_block_height = db.get_latest_block_height();

        let block_height = latest_block_height.unwrap_or(0);
        block_height
    }

    pub fn mine_block(
        &self,
        mut block_count: u64,
        timestamp: u64,
        hash: B256,
    ) -> Result<(), &'static str> {
        self.require_no_waiting_txes()?;

        let mut number = self.get_next_block_height();

        if self.get_block_by_number(0, false).is_none() {
            #[cfg(debug_assertions)]
            println!("Mining genesis block");

            let genesis_hash = B256::ZERO;
            let genesis_timestamp = timestamp;
            let genesis_height = 0;

            self.finalise_block(genesis_timestamp, genesis_height, genesis_hash, 0)?;
            block_count -= 1;
            number += 1;
        }

        #[cfg(debug_assertions)]
        println!(
            "Mining blocks from 0x{:x} ({}) to 0x{:x} ({})",
            number,
            number,
            number + block_count - 1,
            number + block_count - 1
        );

        for _ in 0..block_count {
            self.finalise_block(timestamp, number, hash, 0)?;
            number += 1;
        }

        Ok(())
    }

    pub fn get_contract_address_by_inscription_id(
        &self,
        inscription_id: String,
    ) -> Result<Address, &'static str> {
        #[cfg(debug_assertions)]
        println!(
            "Getting contract address by inscription id {:?}",
            inscription_id
        );

        let mut db = self.db_mutex.lock().unwrap();
        let tx_hash = db
            .get_tx_hash_by_inscription_id(inscription_id)
            .unwrap_or(None);
        if tx_hash.is_none() {
            return Err("inscription not found");
        }
        let tx_hash = tx_hash.unwrap().0;
        let tx = db.get_tx_receipt(tx_hash).unwrap_or(None);
        if tx.is_none() {
            return Err("inscription transaction not found");
        }
        let tx = tx.unwrap();
        if tx.contract_address.is_none() {
            return Err("inscription transaction is not a contract creation");
        }
        Ok(tx.contract_address.unwrap().0)
    }

    pub fn add_tx_to_block(
        &self,
        timestamp: u64,
        tx_info: &TxInfo,
        tx_idx: u64,
        block_number: u64,
        block_hash: B256,
        inscription_id: Option<String>,
        inscription_byte_len: Option<u64>,
    ) -> Result<TxReceiptED, &'static str> {
        #[cfg(debug_assertions)]
        println!("Adding tx {:?} to block {:?}", tx_idx, block_number);

        let mut last_block_info = self.last_block_info.lock().unwrap();

        if last_block_info.waiting_tx_count != tx_idx {
            return Err("tx_idx is different from waiting tx count in block");
        }
        if last_block_info.waiting_tx_count != 0 {
            if timestamp != last_block_info.last_ts {
                return Err("Timestamp is different from other txes in block");
            }

            if block_hash != last_block_info.last_block_hash {
                return Err("Block hash is different from other txes in block");
            }
        } else {
            *last_block_info = LastBlockInfo {
                waiting_tx_count: 0,
                last_ts: timestamp,
                last_block_hash: block_hash,
                last_block_gas_used: 0,
                last_block_log_index: 0,
                last_block_start_time: Instant::now().into(),
            };
        }

        let block_info: BlockEnv = BlockEnv {
            number: block_number,
            timestamp: timestamp,
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

            let start_time = Instant::now();

            evm.ctx().modify_tx(|tx| {
                tx.chain_id = Some(331337);
                tx.caller = tx_info.from;
                tx.kind = tx_info
                    .to
                    .map(|x| TransactTo::Call(x))
                    .unwrap_or(TransactTo::Create);
                tx.data = tx_info.data.clone();
                tx.nonce = nonce;
                tx.gas_limit = get_gas_limit(inscription_byte_len.unwrap_or(tx.data.len() as u64));
            });

            let tx = evm.ctx().tx().clone();
            output = Some(evm.transact_commit(tx).unwrap());

            println!(
                "Tx 0x{:x} ({}) took {}ms",
                tx_idx,
                tx_idx,
                start_time.elapsed().as_millis()
            );
            core::mem::swap(&mut *db, &mut evm.ctx().db());
        }

        let output = output.unwrap();

        last_block_info.waiting_tx_count += 1;

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

        last_block_info.last_block_gas_used = last_block_info
            .last_block_gas_used
            .checked_add(output.gas_used())
            .unwrap_or(last_block_info.last_block_gas_used);

        let mut db = self.db_mutex.lock().unwrap();
        db.set_tx_receipt(
            &get_result_type(&output),
            &get_result_reason(&output),
            output.output(),
            block_hash,
            block_number,
            timestamp,
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
            inscription_id,
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

    pub fn get_transaction_receipt_by_inscription_id(
        &self,
        inscription_id: String,
    ) -> Option<TxReceiptED> {
        #[cfg(debug_assertions)]
        println!("Getting tx receipt by inscription id {:?}", inscription_id);

        let mut db = self.db_mutex.lock().unwrap();
        let tx_hash = db
            .get_tx_hash_by_inscription_id(inscription_id)
            .unwrap_or(None);
        if tx_hash.is_none() {
            return None;
        }
        db.get_tx_receipt(tx_hash.unwrap().0).unwrap_or(None)
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
        block_tx_count: u64,
    ) -> Result<(), &'static str> {
        let mut last_block_info = self.last_block_info.lock().unwrap();

        if last_block_info.last_block_start_time.is_none() {
            last_block_info.last_block_start_time = Some(Instant::now());
        }

        if last_block_info.waiting_tx_count != 0 {
            if timestamp != last_block_info.last_ts {
                return Err("Timestamp is different from other txes in block");
            }

            if block_hash != last_block_info.last_block_hash {
                return Err("Block hash is different from other txes in block");
            }
        }
        if last_block_info.waiting_tx_count != block_tx_count {
            return Err("Block tx count is different from waiting tx count for block");
        }

        let mut db = self.db_mutex.lock().unwrap();

        #[cfg(debug_assertions)]
        println!(
            "Finalising block 0x{:x} ({}), tx count: 0x{:x} ({})",
            block_number, block_number, block_tx_count, block_tx_count
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

        let number = self.get_next_block_height();

        let timestamp = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs();
        let block_info: BlockEnv = BlockEnv {
            number,
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

            evm.ctx().modify_tx(|tx| {
                tx.chain_id = Some(331337);
                tx.caller = tx_info.from;
                tx.kind = tx_info
                    .to
                    .map(|x| TransactTo::Call(x))
                    .unwrap_or(TransactTo::Create);
                tx.data = tx_info.data.clone();
                tx.nonce = nonce;
            });

            let tx = evm.ctx().tx().clone();

            output = evm.transact(tx).map(|x| x.result);
            core::mem::swap(&mut *db, &mut evm.ctx().db());
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
            block_timestamp: timestamp,
            transaction_hash: B256ED::from_b256(txhash),
            transaction_index: 0,
            cumulative_gas_used: output.as_ref().unwrap().gas_used(),
            nonce,
            effective_gas_price: 0,
            transaction_type: 0,
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

    pub fn get_block_by_number(&self, block_number: u64, is_full: bool) -> Option<BlockResponseED> {
        #[cfg(debug_assertions)]
        println!(
            "Getting block by number 0x{:x} ({})",
            block_number, block_number
        );

        let mut db = self.db_mutex.lock().unwrap();
        let block = db.get_block(block_number).unwrap();
        if block.is_none() || !is_full {
            return block;
        }
        // Fill in transaction receipts
        let mut block = block.unwrap();
        let tx_ids = block.transactions.unwrap_or(vec![]);
        let mut tx_receipts = Vec::new();
        for tx_id in tx_ids {
            let tx_receipt = db.get_tx_receipt(tx_id.0);
            if tx_receipt.is_err() {
                continue;
            }
            let tx_receipt = tx_receipt.unwrap();
            if tx_receipt.is_none() {
                continue;
            }
            tx_receipts.insert(tx_receipts.len(), tx_receipt.unwrap());
        }
        block.full_transactions = Some(tx_receipts);
        block.transactions = None;
        Some(block)
    }

    pub fn get_block_by_hash(&self, block_hash: B256, is_full: bool) -> Option<BlockResponseED> {
        #[cfg(debug_assertions)]
        println!("Getting block by hash {:?}", block_hash);

        let mut db = self.db_mutex.lock().unwrap();
        let block_number = db.get_block_number(block_hash).unwrap();

        if block_number.is_none() {
            return None;
        }

        #[cfg(debug_assertions)]
        println!("Got block {:?} hash {:?}", block_number, block_hash);

        self.get_block_by_number(block_number.unwrap().to_u64(), is_full)
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

        let current_block_height = self.get_latest_block_height();
        if latest_valid_block_number > current_block_height {
            return Err("Latest valid block number is greater than current block height");
        }
        if current_block_height - latest_valid_block_number > MAX_HISTORY_SIZE {
            return Err("Latest valid block number is too far behind current block height");
        }
        if latest_valid_block_number == current_block_height {
            return Ok(());
        }

        let mut db = self.db_mutex.lock().unwrap();
        db.reorg(latest_valid_block_number).unwrap();
        Ok(())
    }

    fn require_no_waiting_txes(&self) -> Result<(), &'static str> {
        let last_block_info = self.last_block_info.lock().unwrap();
        if last_block_info.waiting_tx_count != 0 {
            return Err("There are waiting txes, either finalise the block or clear caches");
        }
        Ok(())
    }

    fn get_nonce(&self, addr: Address) -> u64 {
        let mut db = self.db_mutex.lock().unwrap();
        db.basic(addr).unwrap().map_or(0, |x| x.nonce)
    }
}
