use std::error::Error;
use std::time::{Instant, UNIX_EPOCH};

use revm::context::{BlockEnv, ContextTr, TransactTo};
use revm::handler::{EvmTr, ExecuteCommitEvm};
use revm::primitives::alloy_primitives::logs_bloom;
use revm::primitives::{Address, B256, U256};
use revm::ExecuteEvm;

use crate::brc20_controller::{load_brc20_deploy_tx, verify_brc20_contract_address};
use crate::db::types::{
    AddressED, BlockResponseED, BytecodeED, Decode, LogED, LogResponse, TxED, TxReceiptED, B2048ED,
    B256ED,
};
use crate::db::{DB, MAX_HISTORY_SIZE};
use crate::evm::get_evm;
use crate::evm::precompiles::get_brc20_balance;
use crate::evm::utils::{get_contract_address, get_gas_limit, get_result_reason, get_result_type};
use crate::server::shared_data::SharedData;
use crate::server::types::{get_tx_hash, LastBlockInfo, TxInfo};

pub struct BRC20ProgEngine {
    pub db: SharedData<DB>,
    pub last_block_info: SharedData<LastBlockInfo>,
}

impl BRC20ProgEngine {
    pub fn new(db: DB) -> Self {
        let engine = BRC20ProgEngine {
            db: SharedData::new(db),
            last_block_info: SharedData::new(LastBlockInfo::new()),
        };

        engine
    }

    pub fn initialise(
        &self,
        mut genesis_hash: B256,
        genesis_timestamp: u64,
        genesis_height: u64,
    ) -> Result<(), Box<dyn Error>> {
        if genesis_hash == B256::ZERO {
            genesis_hash = generate_block_hash(genesis_height);
        }

        if let Some(genesis) = self.get_block_by_number(genesis_height, false)? {
            if genesis.hash.0 == genesis_hash {
                return Ok(());
            } else {
                return Err("Genesis block hash mismatch".into());
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

        let brc20_controller_contract = result
            .contract_address
            .ok_or("Failed to deploy BRC20_Controller")?;

        verify_brc20_contract_address(&brc20_controller_contract.0.to_string())
            .map_err(|_| "Invalid BRC20_Controller contract address")?;

        self.finalise_block(genesis_timestamp, genesis_height, genesis_hash, 1)?;

        // Check status of BRC20 Balance Server before proceeding
        get_brc20_balance(&[10].into(), &[10].into())
            .map_err(|_| "BRC20 Balance Server is down. This error can be ignored in tests that doesn't involve the BRC20 indexer.")?;

        Ok(())
    }

    pub fn get_next_block_height(&self) -> Result<u64, Box<dyn Error>> {
        let block_height = self.get_latest_block_height()?;
        if block_height == 0 {
            // Check if block 0 exists, if not, next block would be genesis (block 0)
            if self.get_block_by_number(0, false)?.is_some() {
                return Ok(1);
            } else {
                return Ok(0);
            }
        }

        Ok(block_height + 1)
    }

    pub fn get_latest_block_height(&self) -> Result<u64, Box<dyn Error>> {
        self.db.read().get_latest_block_height()
    }

    pub fn mine_blocks(&self, mut block_count: u64, timestamp: u64) -> Result<(), Box<dyn Error>> {
        self.require_no_waiting_txes()?;

        let mut block_number = self.get_next_block_height()?;

        if self.get_block_by_number(0, false)?.is_none() {
            let genesis_hash = B256::ZERO;
            let genesis_timestamp = timestamp;
            let genesis_height = 0;

            self.finalise_block(genesis_timestamp, genesis_height, genesis_hash, 0)?;
            block_count -= 1;
            block_number += 1;
        }

        for _ in 0..block_count {
            self.finalise_block(timestamp, block_number, B256::ZERO, 0)?;
            block_number += 1;
        }

        Ok(())
    }

    pub fn get_contract_address_by_inscription_id(
        &self,
        inscription_id: String,
    ) -> Result<Option<Address>, Box<dyn Error>> {
        self.db.read_fn(|db| {
            let Some(tx_hash) = db.get_tx_hash_by_inscription_id(inscription_id)? else {
                return Ok(None);
            };
            let Some(receipt) = db.get_tx_receipt(tx_hash.0)? else {
                return Ok(None);
            };

            Ok(receipt.contract_address.map(|x| x.0))
        })
    }

    pub fn add_tx_to_block(
        &self,
        timestamp: u64,
        tx_info: &TxInfo,
        tx_idx: u64,
        block_number: u64,
        mut block_hash: B256,
        inscription_id: Option<String>,
        inscription_byte_len: Option<u64>,
    ) -> Result<TxReceiptED, Box<dyn Error>> {
        // This allows testing, and generating hashes for blocks with unknown hashes
        if block_hash == B256::ZERO {
            block_hash = generate_block_hash(block_number);
        }

        self.validate_next_tx(tx_idx, block_hash, block_number, timestamp)?;

        // First tx in block, set last block info
        if self.last_block_info.read().waiting_tx_count == 0 {
            self.last_block_info.write_fn(|block_info| {
                *block_info = LastBlockInfo {
                    waiting_tx_count: 0,
                    timestamp,
                    hash: block_hash,
                    gas_used: 0,
                    log_index: 0,
                    start_time: Instant::now().into(),
                };
                Ok(())
            })?;
        }

        let block_info: BlockEnv = BlockEnv {
            number: block_number,
            timestamp: timestamp,
            ..Default::default()
        };

        let nonce = self.get_nonce(tx_info.from)?;
        let tx_hash = get_tx_hash(&tx_info, &nonce);
        let gas_limit = get_gas_limit(inscription_byte_len.unwrap_or(tx_info.data.len() as u64));

        let output = self.db.write_fn(|db| {
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
                tx.gas_limit = gas_limit;
            });

            let tx = evm.ctx().tx().clone();
            let output = evm.transact_commit(tx)?;

            core::mem::swap(&mut *db, &mut evm.ctx().db());
            Ok(output)
        })?;

        self.last_block_info.write_fn(|last_block_info| {
            last_block_info.waiting_tx_count += 1;
            last_block_info.gas_used = last_block_info
                .gas_used
                .checked_add(output.gas_used())
                .unwrap_or(last_block_info.gas_used);
            last_block_info.log_index += output.logs().len() as u64;
            Ok(())
        })?;

        self.db.write_fn(|db| {
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
                tx_hash,
                tx_idx,
                &output.clone(),
                self.last_block_info.read().gas_used,
                nonce,
                self.last_block_info.read().log_index,
                inscription_id,
                gas_limit,
            )?;

            db.get_tx_receipt(tx_hash)?
                .ok_or("Failed to set tx receipt".into())
        })
    }

    pub fn get_transaction_count(
        &self,
        account: Address,
        block_number: u64,
    ) -> Result<u64, Box<dyn Error>> {
        self.db.read().get_tx_count(Some(account), block_number)
    }

    pub fn get_block_transaction_count_by_number(
        &self,
        block_number: u64,
    ) -> Result<u64, Box<dyn Error>> {
        self.db.read().get_tx_count(None, block_number)
    }

    pub fn get_block_transaction_count_by_hash(
        &self,
        block_hash: B256,
    ) -> Result<u64, Box<dyn Error>> {
        self.db.read_fn(|db| {
            let block_number = db
                .get_block_number(block_hash)?
                .ok_or("Block not found")?
                .to_u64();

            db.get_tx_count(None, block_number)
        })
    }

    pub fn get_transaction_by_block_hash_and_index(
        &self,
        block_hash: B256,
        tx_idx: u64,
    ) -> Result<Option<TxED>, Box<dyn Error>> {
        self.db.read_fn(|db| {
            db.get_tx_hash_by_block_hash_and_index(block_hash, tx_idx)?
                .map_or(Ok(None), |tx_hash| db.get_tx_by_hash(tx_hash.0))
        })
    }

    pub fn get_transaction_by_block_number_and_index(
        &self,
        block_number: u64,
        tx_idx: u64,
    ) -> Result<Option<TxED>, Box<dyn Error>> {
        self.db.read_fn(|db| {
            db.get_tx_hash_by_block_number_and_index(block_number, tx_idx)?
                .map_or(Ok(None), |tx_hash| db.get_tx_by_hash(tx_hash.0))
        })
    }

    pub fn get_transaction_by_hash(&self, tx_hash: B256) -> Result<Option<TxED>, Box<dyn Error>> {
        self.db.read().get_tx_by_hash(tx_hash)
    }

    pub fn get_transaction_receipt_by_inscription_id(
        &self,
        inscription_id: String,
    ) -> Result<Option<TxReceiptED>, Box<dyn Error>> {
        self.db.read_fn(|db| {
            db.get_tx_hash_by_inscription_id(inscription_id)?
                .map_or(Ok(None), |tx_hash| db.get_tx_receipt(tx_hash.0))
        })
    }

    pub fn get_transaction_receipt(
        &self,
        tx_hash: B256,
    ) -> Result<Option<TxReceiptED>, Box<dyn Error>> {
        self.db.read().get_tx_receipt(tx_hash)
    }

    pub fn get_logs(
        &self,
        block_number_from: Option<u64>,
        block_number_to: Option<u64>,
        address: Option<Address>,
        topics: Option<Vec<B256>>,
    ) -> Result<Vec<LogResponse>, Box<dyn Error>> {
        self.db.read().get_logs(
            block_number_from,
            block_number_to,
            address,
            topics.unwrap_or(Vec::new()),
        )
    }

    pub fn finalise_block(
        &self,
        timestamp: u64,
        block_number: u64,
        mut block_hash: B256,
        block_tx_count: u64,
    ) -> Result<(), Box<dyn Error>> {
        // This allows testing, and generating hashes for blocks with unknown hashes
        if block_hash == B256::ZERO {
            block_hash = generate_block_hash(block_number);
        }

        self.validate_next_tx(block_tx_count, block_hash, block_number, timestamp)?;

        self.db.write_fn(|db| {
            let (total_time_took, gas_used) = self.last_block_info.read_fn(|info| {
                let total_time_took = info.start_time.map(|x| x.elapsed().as_nanos()).unwrap_or(0);
                Ok((total_time_took, info.gas_used))
            })?;

            db.set_block_hash(block_number, block_hash)?;
            db.set_mine_timestamp(block_number, total_time_took)?;
            db.set_gas_used(block_number, gas_used)?;
            db.set_block_timestamp(block_number, timestamp)?;

            // Save the full block info in the database for ease of access
            db.set_block(block_number, db.generate_block(block_number)?)
        })?;

        self.last_block_info.write_fn_unchecked(|last_block_info| {
            *last_block_info = LastBlockInfo::new();
        });

        Ok(())
    }

    pub fn call_contract(&self, tx_info: &TxInfo) -> Result<TxReceiptED, Box<dyn Error>> {
        self.require_no_waiting_txes()?;

        let block_number = self.get_next_block_height()?;
        let timestamp = UNIX_EPOCH.elapsed().map(|x| x.as_secs())?;
        let nonce = self.get_nonce(tx_info.from)?;
        let txhash = get_tx_hash(&tx_info, &nonce);
        let block_info: BlockEnv = BlockEnv {
            number: block_number,
            timestamp,
            ..Default::default()
        };

        // This isn't actually writing to the database, but the EVM context requires a mutable reference
        let output = self.db.write_fn(|db| {
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
                // Use 10x gas limit for call
                tx.gas_limit = get_gas_limit(tx_info.data.len() as u64) * 10;
            });

            let tx = evm.ctx().tx().clone();
            let output = evm.transact(tx).map(|x| x.result);

            core::mem::swap(&mut *db, &mut evm.ctx().db());

            output.map_err(|e| e.into())
        })?;

        Ok(TxReceiptED {
            status: output.is_success() as u8,
            transaction_result: get_result_type(&output),
            reason: get_result_reason(&output),
            result_bytes: output.output().cloned(),
            logs: LogED {
                logs: output.logs().to_vec(),
                log_index: 0,
            },
            log_responses: LogResponse::new_vec(
                &LogED {
                    logs: output.logs().to_vec(),
                    log_index: 0,
                },
                0,
                B256ED::from_b256(txhash),
                B256ED::from_b256(txhash),
                block_number,
            ),
            gas_used: output.gas_used(),
            from: AddressED(tx_info.from),
            to: tx_info.to.map(AddressED),
            contract_address: get_contract_address(&output).map(AddressED),
            logs_bloom: B2048ED::decode(logs_bloom(output.logs()).to_vec())
                .map_err(|_| "Error while decoding logs bloom")?,
            hash: B256ED::from_b256(txhash),
            block_number,
            block_timestamp: timestamp,
            transaction_hash: B256ED::from_b256(txhash),
            transaction_index: 0,
            cumulative_gas_used: output.gas_used(),
            nonce,
            effective_gas_price: 0,
            transaction_type: 0,
        })
    }

    pub fn get_storage_at(
        &self,
        contract: Address,
        location: U256,
    ) -> Result<U256, Box<dyn Error>> {
        Ok(self
            .db
            .read()
            .get_account_memory(contract, location)?
            .map(|x| x.0)
            .unwrap_or(U256::ZERO))
    }

    pub fn get_block_by_number(
        &self,
        block_number: u64,
        is_full: bool,
    ) -> Result<Option<BlockResponseED>, Box<dyn Error>> {
        self.db.read_fn(|db| {
            db.get_block(block_number)?.map_or(Ok(None), |mut block| {
                if !is_full {
                    return Ok(Some(block));
                }
                let tx_ids = block.transactions.unwrap_or(vec![]);
                let mut txes = Vec::new();
                for tx_id in tx_ids {
                    let Some(tx) = db.get_tx_by_hash(tx_id.0)? else {
                        continue;
                    };
                    txes.insert(txes.len(), tx);
                }
                block.full_transactions = Some(txes);
                block.transactions = None;
                Ok(Some(block))
            })
        })
    }

    pub fn get_block_by_hash(
        &self,
        block_hash: B256,
        is_full: bool,
    ) -> Result<Option<BlockResponseED>, Box<dyn Error>> {
        self.db.read_fn(|db| {
            db.get_block_number(block_hash)?
                .map_or(Ok(None), |block_number| {
                    self.get_block_by_number(block_number.to_u64(), is_full)
                })
        })
    }

    pub fn get_contract_bytecode(
        &self,
        addr: Address,
    ) -> Result<Option<BytecodeED>, Box<dyn Error>> {
        self.db.read_fn(|db| {
            db.get_account_info(addr)?
                .map_or(Ok(None), |acct| db.get_code(acct.0.code_hash))
        })
    }

    pub fn clear_caches(&self) -> Result<(), Box<dyn Error>> {
        self.last_block_info.write_fn_unchecked(|last_block_info| {
            *last_block_info = LastBlockInfo::new();
        });

        self.db.write_fn(|db| db.clear_caches())
    }

    pub fn commit_to_db(&self) -> Result<(), Box<dyn Error>> {
        self.require_no_waiting_txes()?;

        self.db.write_fn(|db| db.commit_changes())
    }

    pub fn reorg(&self, latest_valid_block_number: u64) -> Result<(), Box<dyn Error>> {
        self.require_no_waiting_txes()?;

        let current_block_height = self.get_latest_block_height()?;
        if latest_valid_block_number > current_block_height {
            return Err("Latest valid block number is greater than current block height".into());
        }
        if current_block_height - latest_valid_block_number > MAX_HISTORY_SIZE {
            return Err("Latest valid block number is too far behind current block height".into());
        }
        if latest_valid_block_number == current_block_height {
            return Ok(());
        }

        self.db.write_fn(|db| db.reorg(latest_valid_block_number))
    }

    fn require_no_waiting_txes(&self) -> Result<(), Box<dyn Error>> {
        if self.last_block_info.read().waiting_tx_count != 0 {
            return Err("There are waiting txes, either finalise the block or clear caches".into());
        }
        Ok(())
    }

    fn validate_next_tx(
        &self,
        tx_idx: u64,
        block_hash: B256,
        block_number: u64,
        timestamp: u64,
    ) -> Result<(), Box<dyn Error>> {
        self.last_block_info.read_fn(|info| {
            if info.waiting_tx_count != tx_idx {
                return Err("tx_idx is different from waiting tx count in block".into());
            }
            if info.waiting_tx_count != 0 {
                if info.timestamp != timestamp {
                    return Err("Timestamp is different from other txes in block".into());
                }
                if info.hash != block_hash {
                    return Err("Block hash is different from other txes in block".into());
                }
            }
            Ok(())
        })?;
        self.db
            .read()
            .require_block_does_not_exist(block_hash, block_number)
    }

    fn get_nonce(&self, addr: Address) -> Result<u64, Box<dyn Error>> {
        Ok(self
            .db
            .read()
            .get_account_info(addr)?
            .map(|x| x.0.nonce)
            .unwrap_or(0))
    }
}

fn generate_block_hash(block_number: u64) -> B256 {
    // +1 to avoid zero hash
    let bytes = (block_number + 1).to_be_bytes();
    let full_bytes = [0u8; 24]
        .iter()
        .chain(bytes.iter())
        .copied()
        .collect::<Vec<u8>>();
    B256::from_slice(&full_bytes)
}
