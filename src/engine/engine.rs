#![cfg(feature = "server")]

use std::error::Error;
use std::time::{Duration, UNIX_EPOCH};

use alloy::consensus::transaction::RlpEcdsaDecodableTx;
use alloy::consensus::{SignableTransaction, TxLegacy};
use alloy::primitives::{keccak256, logs_bloom, Address, B256, U256};
use alloy_rpc_types_trace::geth::CallConfig;
use either::Either::Right;
use revm::context::ContextTr;
use revm::handler::EvmTr;
use revm::inspector::InspectorEvmTr;
use revm::{ExecuteEvm, InspectCommitEvm};
use serde_either::SingleOrVec;

use crate::brc20_controller::{load_brc20_deploy_tx, verify_brc20_contract_address};
use crate::db::types::{
    BlockResponseED, BytecodeED, Decode, LogED, TraceED, TxED, TxReceiptED, B2048ED,
};
use crate::db::Brc20ProgDatabase;
use crate::engine::evm::get_evm;
use crate::engine::precompiles::get_brc20_balance;
use crate::engine::utils::{
    get_contract_address, get_gas_limit, get_result_reason, get_result_type, get_tx_hash,
    LastBlockInfo, TxInfo,
};
use crate::global::{SharedData, CHAIN_ID, CONFIG, MAX_REORG_HISTORY_SIZE};

pub struct BRC20ProgEngine {
    db: SharedData<Brc20ProgDatabase>,
    last_block_info: SharedData<LastBlockInfo>,
}

impl BRC20ProgEngine {
    pub fn new(db: Brc20ProgDatabase) -> Self {
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
            if genesis.hash.bytes == genesis_hash {
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

        verify_brc20_contract_address(&brc20_controller_contract.address.to_string())
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
            let Some(receipt) = db.get_tx_receipt(tx_hash.bytes)? else {
                return Ok(None);
            };

            Ok(receipt.contract_address.map(|x| x.address))
        })
    }

    pub fn add_raw_tx_to_block(
        &self,
        timestamp: u64,
        raw_tx: Vec<u8>,
        tx_idx: u64,
        block_number: u64,
        mut block_hash: B256,
        inscription_id: Option<String>,
        inscription_byte_len: Option<u64>,
    ) -> Result<Option<TxED>, Box<dyn Error>> {
        // This allows testing, and generating hashes for blocks with unknown hashes
        if block_hash == B256::ZERO {
            block_hash = generate_block_hash(block_number);
        }

        let receipt = self.add_tx_to_block(
            timestamp,
            &self.get_info_from_raw_tx(raw_tx.clone())?,
            tx_idx,
            block_number,
            block_hash,
            inscription_id,
            inscription_byte_len,
        )?;

        Ok(self.get_transaction_by_hash(receipt.transaction_hash.bytes)?)
    }

    pub fn get_info_from_raw_tx(&self, mut raw_tx: Vec<u8>) -> Result<TxInfo, Box<dyn Error>> {
        let (decoded_raw_tx, signature) =
            TxLegacy::rlp_decode_with_signature(&mut raw_tx.as_mut_slice().as_ref())
                .map_err(|_| "Failed to decode legacy transaction")?;

        if decoded_raw_tx.chain_id != Some(CHAIN_ID) {
            return Err(format!(
                "Invalid chain ID in raw transaction: {:?}",
                decoded_raw_tx.chain_id
            )
            .into());
        }

        let signing_hash = keccak256(decoded_raw_tx.encoded_for_signing());
        let recovered_address = signature.recover_address_from_prehash(&signing_hash)?;

        Ok(TxInfo::from_raw_transaction(
            recovered_address,
            decoded_raw_tx,
            signing_hash,
        ))
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

        // TODO: Record nonces from the future, so we can execute them later
        // This is useful for reorgs, where we might have txes with nonces that are not in the current block
        // We can store them in a separate table, and execute them when the nonce is available
        let account_nonce = self.get_account_nonce(tx_info.from)?;
        if let Some(nonce) = tx_info.nonce {
            if nonce != account_nonce {
                return Err("Nonce is different from account nonce".into());
            }
        }

        let tx_nonce = tx_info.nonce.unwrap_or(account_nonce);

        self.validate_next_tx(tx_idx, block_hash, block_number, timestamp)?;

        // First tx in block, set last block info
        if self.last_block_info.read().waiting_tx_count == 0 {
            self.last_block_info.write_fn_unchecked(|block_info| {
                *block_info = LastBlockInfo {
                    waiting_tx_count: 0,
                    timestamp,
                    hash: block_hash,
                    gas_used: 0,
                    log_index: 0,
                    start_time: block_info.start_time,
                    total_processing_time: None,
                };
            });
        }

        let tx_hash = get_tx_hash(&tx_info, account_nonce);
        let gas_limit = get_gas_limit(inscription_byte_len.unwrap_or(tx_info.data.len() as u64));

        self.db.write_fn(|db| {
            let processing_start_time = self.last_block_info.read().start_time.elapsed();

            let db_moved = core::mem::take(&mut *db);
            let mut evm = get_evm(block_number, block_hash, timestamp, db_moved, None);

            evm.ctx().modify_tx(|tx| {
                tx.caller = tx_info.from;
                tx.kind = tx_info.to;
                tx.data = tx_info.data.clone();
                tx.nonce = tx_nonce;
                tx.gas_limit = gas_limit;
            });

            let output = evm.inspect_replay_commit()?;

            core::mem::swap(&mut *db, &mut evm.ctx().db());

            let cumulative_gas_used = self
                .last_block_info
                .read()
                .gas_used
                .checked_add(output.gas_used())
                .unwrap_or(self.last_block_info.read().gas_used);

            let traces: TraceED = evm
                .inspector()
                .geth_builder()
                .geth_call_traces(
                    CallConfig {
                        only_top_call: Some(false),
                        with_log: Some(true),
                    },
                    output.gas_used(),
                )
                .into();

            if let Some(inscription_id) = inscription_id.clone() {
                // If this is a contract creation with inscription id, store it
                if let Some(created_contract) = traces.get_created_contract() {
                    db.set_contract_address_to_inscription_id(
                        created_contract.address,
                        inscription_id.clone(),
                    )?;
                }
            }

            if CONFIG.read().evm_record_traces {
                db.set_tx_trace(tx_hash, traces)?;
            }

            db.set_tx_receipt(
                &get_result_type(&output),
                &get_result_reason(&output),
                output.output(),
                block_hash,
                block_number,
                timestamp,
                get_contract_address(&output),
                tx_info.from,
                tx_info.to_address_optional(),
                &tx_info.data,
                tx_hash,
                tx_idx,
                &output.clone(),
                cumulative_gas_used,
                tx_nonce,
                self.last_block_info.read().log_index,
                inscription_id,
                gas_limit,
            )?;

            self.last_block_info.write_fn_unchecked(|last_block_info| {
                last_block_info.waiting_tx_count += 1;
                last_block_info.gas_used = last_block_info
                    .gas_used
                    .checked_add(output.gas_used())
                    .unwrap_or(last_block_info.gas_used);
                last_block_info.log_index += output.logs().len() as u64;
                last_block_info.total_processing_time = Some(
                    (last_block_info.start_time.elapsed() - processing_start_time)
                        + last_block_info
                            .total_processing_time
                            .unwrap_or(Duration::ZERO),
                );
            });

            db.get_tx_receipt(tx_hash)?
                .ok_or("Failed to set tx receipt".into())
        })
    }

    pub fn get_inscription_id_by_contract_address(
        &self,
        contract_address: Address,
    ) -> Result<Option<String>, Box<dyn Error>> {
        self.db
            .read()
            .get_inscription_id_by_contract_address(contract_address)
    }

    pub fn get_trace(&self, tx_hash: B256) -> Result<Option<TraceED>, Box<dyn Error>> {
        self.db.read().get_tx_trace(tx_hash)
    }

    pub fn get_transaction_count(
        &self,
        account: Address,
        _block_number: u64,
    ) -> Result<u64, Box<dyn Error>> {
        let account_nonce: u64 = self
            .db
            .read()
            .get_account_info(account)?
            .map(|x| x.nonce.into())
            .unwrap_or(0);

        Ok(account_nonce)
    }

    pub fn get_block_transaction_count_by_number(
        &self,
        block_number: u64,
    ) -> Result<u64, Box<dyn Error>> {
        self.db.read().get_block_tx_count(block_number)
    }

    pub fn get_block_transaction_count_by_hash(
        &self,
        block_hash: B256,
    ) -> Result<u64, Box<dyn Error>> {
        self.db.read_fn(|db| {
            let block_number = db
                .get_block_number(block_hash)?
                .ok_or("Block not found")?
                .into();

            db.get_block_tx_count(block_number)
        })
    }

    pub fn get_transaction_by_block_hash_and_index(
        &self,
        block_hash: B256,
        tx_idx: u64,
    ) -> Result<Option<TxED>, Box<dyn Error>> {
        self.db.read_fn(|db| {
            db.get_tx_hash_by_block_hash_and_index(block_hash, tx_idx)?
                .map_or(Ok(None), |tx_hash| db.get_tx_by_hash(tx_hash.bytes))
        })
    }

    pub fn get_transaction_by_block_number_and_index(
        &self,
        block_number: u64,
        tx_idx: u64,
    ) -> Result<Option<TxED>, Box<dyn Error>> {
        self.db.read_fn(|db| {
            db.get_tx_hash_by_block_number_and_index(block_number, tx_idx)?
                .map_or(Ok(None), |tx_hash| db.get_tx_by_hash(tx_hash.bytes))
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
                .map_or(Ok(None), |tx_hash| db.get_tx_receipt(tx_hash.bytes))
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
        topics: Option<Vec<SingleOrVec<Option<B256>>>>,
    ) -> Result<Vec<LogED>, Box<dyn Error>> {
        self.db
            .read()
            .get_logs(block_number_from, block_number_to, address, topics)
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
                let total_time_took = info
                    .total_processing_time
                    .unwrap_or(Duration::ZERO)
                    .as_nanos();
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

    pub fn read_contract(&self, tx_info: &TxInfo) -> Result<TxReceiptED, Box<dyn Error>> {
        self.require_no_waiting_txes()?;

        let block_number = self.get_next_block_height()?;
        let timestamp = UNIX_EPOCH.elapsed().map(|x| x.as_secs())?;
        let nonce = self.get_account_nonce(tx_info.from)?;
        let txhash = get_tx_hash(&tx_info, nonce);

        // This isn't actually writing to the database, but the EVM context requires a mutable reference
        let output = self.db.write_fn(|db| {
            let db_moved = core::mem::take(&mut *db);
            let mut evm = get_evm(block_number, B256::ZERO, timestamp, db_moved, None);

            evm.ctx().modify_tx(|tx| {
                tx.caller = tx_info.from;
                tx.kind = tx_info.to;
                tx.data = tx_info.data.clone();
                tx.nonce = nonce;
                tx.gas_limit = CONFIG.read().evm_call_gas_limit;
            });

            let output = evm.replay().map(|x| x.result);
            core::mem::swap(&mut *db, &mut evm.ctx().db());

            output.map_err(|e| e.into())
        })?;

        Ok(TxReceiptED {
            status: (output.is_success() as u8).into(),
            transaction_result: get_result_type(&output),
            reason: get_result_reason(&output),
            result_bytes: output.output().cloned().map(|x| x.into()),
            logs: LogED::new_vec(
                &output.logs().to_vec(),
                0u64.into(),
                0u64.into(),
                txhash.into(),
                txhash.into(),
                block_number.into(),
            ),
            gas_used: output.gas_used().into(),
            from: tx_info.from.into(),
            to: tx_info.to_address_optional().map(|x| x.into()),
            contract_address: get_contract_address(&output).map(|x| x.into()),
            logs_bloom: B2048ED::decode_vec(&logs_bloom(output.logs()).to_vec())
                .map_err(|_| "Error while decoding logs bloom")?,
            block_hash: txhash.into(),
            block_number: block_number.into(),
            block_timestamp: timestamp.into(),
            transaction_hash: txhash.into(),
            transaction_index: 0u64.into(),
            cumulative_gas_used: output.gas_used().into(),
            nonce: nonce.into(),
            effective_gas_price: 0u64.into(),
            transaction_type: 0u8.into(),
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
            .map(|x| x.uint)
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
                let tx_ids = block.transactions.left().unwrap_or(vec![]);
                let mut txes = Vec::new();
                for tx_id in tx_ids {
                    let Some(tx) = db.get_tx_by_hash(tx_id.bytes)? else {
                        continue;
                    };
                    txes.insert(txes.len(), tx);
                }
                block.transactions = Right(txes);
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
                    self.get_block_by_number(block_number.into(), is_full)
                })
        })
    }

    pub fn get_contract_bytecode(
        &self,
        addr: Address,
    ) -> Result<Option<BytecodeED>, Box<dyn Error>> {
        self.db.read_fn(|db| {
            db.get_account_info(addr)?
                .map_or(Ok(None), |acct| db.get_code(acct.code_hash.bytes))
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
        if current_block_height - latest_valid_block_number > MAX_REORG_HISTORY_SIZE {
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

    fn get_account_nonce(&self, addr: Address) -> Result<u64, Box<dyn Error>> {
        Ok(self
            .db
            .read()
            .get_account_info(addr)?
            .map(|x| x.nonce.into())
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

#[cfg(test)]
mod tests {
    use alloy::primitives::B256;
    use revm::primitives::TxKind;
    use tempfile::TempDir;

    use super::*;
    use crate::db::Brc20ProgDatabase;
    use crate::global::INDEXER_ADDRESS;

    #[test]
    fn test_initialise() {
        let temp_dir = TempDir::new().unwrap();
        let db = Brc20ProgDatabase::new(temp_dir.path()).unwrap();
        let engine = BRC20ProgEngine::new(db);
        let genesis_hash = B256::from_slice([1; 32].as_ref());
        let genesis_timestamp = 1622547800;
        let genesis_height = 0;

        let _ = engine.initialise(genesis_hash, genesis_timestamp, genesis_height);

        let block = engine.get_block_by_number(0, true).unwrap();
        assert!(block.is_some());
        let block = block.unwrap();
        assert_eq!(block.number, 0u64.into());
        assert_eq!(block.hash.bytes, genesis_hash);
        assert_eq!(block.timestamp, genesis_timestamp.into());

        let full_transactions = block.transactions.right().unwrap();
        assert_eq!(full_transactions.len(), 1);
        assert_eq!(full_transactions[0].from.address, *INDEXER_ADDRESS);
        assert_eq!(full_transactions[0].to, None);
    }

    #[test]
    fn test_get_next_block_height() {
        let temp_dir = TempDir::new().unwrap();
        let db = Brc20ProgDatabase::new(temp_dir.path()).unwrap();
        let engine = BRC20ProgEngine::new(db);

        assert_eq!(engine.get_next_block_height().unwrap(), 0);
        let _ = engine.initialise(B256::ZERO, 1622547800, 0);
        assert_eq!(engine.get_next_block_height().unwrap(), 1);
    }

    #[test]
    fn test_get_latest_block_height() {
        let temp_dir = TempDir::new().unwrap();
        let db = Brc20ProgDatabase::new(temp_dir.path()).unwrap();
        let engine = BRC20ProgEngine::new(db);

        assert_eq!(engine.get_latest_block_height().unwrap(), 0);
        let _ = engine.initialise(B256::ZERO, 1622547800, 0);
        assert_eq!(engine.get_latest_block_height().unwrap(), 0);
        engine.mine_blocks(123, 1622547800).unwrap();
        assert_eq!(engine.get_latest_block_height().unwrap(), 123);
    }

    #[test]
    fn test_mine_blocks() {
        let temp_dir = TempDir::new().unwrap();
        let db = Brc20ProgDatabase::new(temp_dir.path()).unwrap();
        let engine = BRC20ProgEngine::new(db);

        let _ = engine.initialise(B256::ZERO, 1622547800, 0);
        assert_eq!(engine.get_next_block_height().unwrap(), 1);
        engine.mine_blocks(2, 1622547800).unwrap();
        assert_eq!(engine.get_next_block_height().unwrap(), 3);
    }

    #[test]
    fn test_get_contract_address_by_inscription_id() {
        let temp_dir = TempDir::new().unwrap();
        let db = Brc20ProgDatabase::new(temp_dir.path()).unwrap();
        let engine = BRC20ProgEngine::new(db);

        let inscription_id = "test_inscription_id".to_string();

        let deployed_contract = engine
            .add_tx_to_block(
                1622547800,
                &TxInfo::from_inscription(*INDEXER_ADDRESS, TxKind::Create, vec![].into()),
                0,
                1,
                B256::ZERO,
                Some(inscription_id.clone()),
                Some(1000),
            )
            .unwrap()
            .contract_address
            .unwrap();

        let contract_address = engine
            .get_contract_address_by_inscription_id(inscription_id.clone())
            .unwrap()
            .unwrap();

        assert_eq!(contract_address, deployed_contract.address);
    }

    #[test]
    fn test_add_tx_to_block() {
        let temp_dir = TempDir::new().unwrap();
        let db = Brc20ProgDatabase::new(temp_dir.path()).unwrap();
        let engine = BRC20ProgEngine::new(db);

        let from_address = Address::from_slice([1; 20].as_ref());
        let tx_info = TxInfo::from_inscription(from_address, TxKind::Create, vec![].into());
        let block_number = 1;
        let block_hash = B256::ZERO;
        let timestamp = 1622547800;

        let result = engine
            .add_tx_to_block(
                timestamp,
                &tx_info,
                0,
                block_number,
                block_hash,
                None,
                Some(1000),
            )
            .unwrap();

        assert_eq!(result.from.address, from_address);
    }

    #[test]
    fn test_get_transaction_count() {
        let temp_dir = TempDir::new().unwrap();
        let db = Brc20ProgDatabase::new(temp_dir.path()).unwrap();
        let engine = BRC20ProgEngine::new(db);

        let account_1 = Address::from_slice([1; 20].as_ref());
        let account_2 = Address::from_slice([2; 20].as_ref());

        engine
            .add_tx_to_block(
                1622547800,
                &TxInfo::from_inscription(account_1, TxKind::Create, vec![].into()),
                0,
                0,
                B256::ZERO,
                None,
                Some(1000),
            )
            .unwrap();

        engine
            .add_tx_to_block(
                1622547800,
                &TxInfo::from_inscription(account_2, TxKind::Create, vec![].into()),
                1,
                0,
                B256::ZERO,
                None,
                Some(1000),
            )
            .unwrap();

        engine.finalise_block(1622547800, 0, B256::ZERO, 2).unwrap();

        assert_eq!(engine.get_transaction_count(account_1, 0).unwrap(), 1);
        assert_eq!(engine.get_transaction_count(account_2, 0).unwrap(), 1);
    }

    #[test]
    fn test_get_block_transaction_count_by_number() {
        let temp_dir = TempDir::new().unwrap();
        let db = Brc20ProgDatabase::new(temp_dir.path()).unwrap();
        let engine = BRC20ProgEngine::new(db);

        engine.mine_blocks(100, 1622547800).unwrap();
        let _ = engine.initialise(B256::ZERO, 1622547800, 100);

        assert_eq!(engine.get_block_transaction_count_by_number(0).unwrap(), 0);

        assert_eq!(engine.get_block_transaction_count_by_number(50).unwrap(), 0);

        assert_eq!(
            engine.get_block_transaction_count_by_number(100).unwrap(),
            1
        );
    }

    #[test]
    fn test_get_block_transaction_count_by_hash() {
        let temp_dir = TempDir::new().unwrap();
        let db = Brc20ProgDatabase::new(temp_dir.path()).unwrap();
        let engine = BRC20ProgEngine::new(db);
        let hash = B256::from_slice([1; 32].as_ref());

        engine
            .add_tx_to_block(
                1622547800,
                &TxInfo::from_inscription(*INDEXER_ADDRESS, TxKind::Create, vec![].into()),
                0,
                0,
                hash,
                None,
                Some(1000),
            )
            .unwrap();

        engine
            .add_tx_to_block(
                1622547800,
                &TxInfo::from_inscription(*INDEXER_ADDRESS, TxKind::Create, vec![].into()),
                1,
                0,
                hash,
                None,
                Some(1000),
            )
            .unwrap();
        engine.finalise_block(1622547800, 0, hash, 2).unwrap();
        assert_eq!(engine.get_block_transaction_count_by_hash(hash).unwrap(), 2);
    }

    #[test]
    fn test_get_transaction_by_block_hash_and_index() {
        let temp_dir = TempDir::new().unwrap();
        let db = Brc20ProgDatabase::new(temp_dir.path()).unwrap();
        let engine = BRC20ProgEngine::new(db);
        let hash = B256::from_slice([1; 32].as_ref());

        let account_1 = Address::from_slice([1; 20].as_ref());
        let account_2 = Address::from_slice([2; 20].as_ref());
        engine
            .add_tx_to_block(
                1622547800,
                &TxInfo::from_inscription(account_1, TxKind::Create, vec![].into()),
                0,
                0,
                hash,
                None,
                Some(1000),
            )
            .unwrap();

        engine
            .add_tx_to_block(
                1622547800,
                &TxInfo::from_inscription(account_2, TxKind::Create, vec![].into()),
                1,
                0,
                hash,
                None,
                Some(1000),
            )
            .unwrap();
        engine.finalise_block(1622547800, 0, hash, 2).unwrap();
        assert_eq!(
            engine
                .get_transaction_by_block_hash_and_index(hash, 0)
                .unwrap()
                .unwrap()
                .from
                .address,
            account_1
        );
        assert_eq!(
            engine
                .get_transaction_by_block_hash_and_index(hash, 1)
                .unwrap()
                .unwrap()
                .from
                .address,
            account_2
        );
    }

    #[test]
    fn test_get_transaction_by_block_number_and_index() {
        let temp_dir = TempDir::new().unwrap();
        let db = Brc20ProgDatabase::new(temp_dir.path()).unwrap();
        let engine = BRC20ProgEngine::new(db);
        let hash = B256::from_slice([1; 32].as_ref());

        let account_1 = Address::from_slice([1; 20].as_ref());
        let account_2 = Address::from_slice([2; 20].as_ref());
        engine
            .add_tx_to_block(
                1622547800,
                &TxInfo::from_inscription(account_1, TxKind::Create, vec![].into()),
                0,
                0,
                hash,
                None,
                Some(1000),
            )
            .unwrap();

        engine
            .add_tx_to_block(
                1622547800,
                &TxInfo::from_inscription(account_2, TxKind::Create, vec![].into()),
                1,
                0,
                hash,
                None,
                Some(1000),
            )
            .unwrap();
        engine.finalise_block(1622547800, 0, hash, 2).unwrap();
        assert_eq!(
            engine
                .get_transaction_by_block_number_and_index(0, 0)
                .unwrap()
                .unwrap()
                .from
                .address,
            account_1
        );
        assert_eq!(
            engine
                .get_transaction_by_block_number_and_index(0, 1)
                .unwrap()
                .unwrap()
                .from
                .address,
            account_2
        );
    }

    #[test]
    fn test_get_transaction_by_hash() {
        let temp_dir = TempDir::new().unwrap();
        let db = Brc20ProgDatabase::new(temp_dir.path()).unwrap();
        let engine = BRC20ProgEngine::new(db);
        let hash = B256::from_slice([1; 32].as_ref());

        let account_1 = Address::from_slice([1; 20].as_ref());
        let tx_hash = engine
            .add_tx_to_block(
                1622547800,
                &TxInfo::from_inscription(account_1, TxKind::Create, vec![].into()),
                0,
                0,
                hash,
                None,
                Some(1000),
            )
            .unwrap()
            .transaction_hash
            .bytes;

        assert_eq!(
            engine
                .get_transaction_by_hash(tx_hash)
                .unwrap()
                .unwrap()
                .from
                .address,
            account_1
        );
    }

    #[test]
    fn test_get_transaction_receipt_by_inscription_id() {
        let temp_dir = TempDir::new().unwrap();
        let db = Brc20ProgDatabase::new(temp_dir.path()).unwrap();
        let engine = BRC20ProgEngine::new(db);

        let inscription_id = "test_inscription_id".to_string();

        let result = engine
            .add_tx_to_block(
                1622547800,
                &TxInfo::from_inscription(*INDEXER_ADDRESS, TxKind::Create, vec![].into()),
                0,
                0,
                B256::ZERO,
                Some(inscription_id.clone()),
                Some(1000),
            )
            .unwrap();

        let receipt = engine
            .get_transaction_receipt_by_inscription_id(inscription_id)
            .unwrap()
            .unwrap();

        assert_eq!(
            receipt.transaction_hash.bytes,
            result.transaction_hash.bytes
        );
    }

    #[test]
    fn test_get_transaction_receipt() {
        let temp_dir = TempDir::new().unwrap();
        let db = Brc20ProgDatabase::new(temp_dir.path()).unwrap();
        let engine = BRC20ProgEngine::new(db);
        let hash = B256::from_slice([1; 32].as_ref());

        let account_1 = Address::from_slice([1; 20].as_ref());
        let tx_hash = engine
            .add_tx_to_block(
                1622547800,
                &TxInfo::from_inscription(account_1, TxKind::Create, vec![].into()),
                0,
                0,
                hash,
                None,
                Some(1000),
            )
            .unwrap()
            .transaction_hash
            .bytes;

        assert_eq!(
            engine
                .get_transaction_receipt(tx_hash)
                .unwrap()
                .unwrap()
                .from
                .address,
            account_1
        );
    }

    #[test]
    fn test_require_no_waiting_txes() {
        let temp_dir = TempDir::new().unwrap();
        let db = Brc20ProgDatabase::new(temp_dir.path()).unwrap();
        let engine = BRC20ProgEngine::new(db);

        assert!(engine.require_no_waiting_txes().is_ok());

        engine
            .add_tx_to_block(
                1622547800,
                &TxInfo::from_inscription(*INDEXER_ADDRESS, TxKind::Create, vec![].into()),
                0,
                0,
                B256::ZERO,
                None,
                Some(10000),
            )
            .unwrap();

        assert!(engine.require_no_waiting_txes().is_err());
    }

    #[test]
    fn test_validate_next_tx() {
        let temp_dir = TempDir::new().unwrap();
        let db = Brc20ProgDatabase::new(temp_dir.path()).unwrap();
        let engine = BRC20ProgEngine::new(db);

        let block_hash = B256::from_slice([1; 32].as_ref());
        let block_number = 0;
        let timestamp = 1622547800;

        assert!(engine
            .validate_next_tx(0, block_hash, block_number, timestamp)
            .is_ok());

        engine
            .add_tx_to_block(
                timestamp,
                &TxInfo::from_inscription(*INDEXER_ADDRESS, TxKind::Create, vec![].into()),
                0,
                block_number,
                block_hash,
                None,
                Some(1000),
            )
            .unwrap();

        assert!(engine
            .validate_next_tx(1, block_hash, block_number, timestamp)
            .is_ok());

        engine
            .add_tx_to_block(
                timestamp,
                &TxInfo::from_inscription(*INDEXER_ADDRESS, TxKind::Create, vec![].into()),
                1,
                block_number,
                block_hash,
                None,
                Some(1000),
            )
            .unwrap();

        assert!(engine
            .validate_next_tx(3, block_hash, block_number, timestamp)
            .is_err());
    }

    #[test]
    fn test_get_nonce() {
        let temp_dir = TempDir::new().unwrap();
        let db = Brc20ProgDatabase::new(temp_dir.path()).unwrap();
        let engine = BRC20ProgEngine::new(db);

        let account = Address::from_slice([1; 20].as_ref());

        assert_eq!(engine.get_account_nonce(account).unwrap(), 0);

        engine
            .add_tx_to_block(
                1622547800,
                &TxInfo::from_inscription(account, TxKind::Create, vec![].into()),
                0,
                0,
                B256::ZERO,
                None,
                Some(1000),
            )
            .unwrap();

        assert_eq!(engine.get_account_nonce(account).unwrap(), 1);
    }

    #[test]
    fn test_generate_block_hash() {
        let block_number = 1;
        let block_hash = generate_block_hash(block_number);
        assert_eq!(block_hash.0[0..30], [0u8; 30]);
        assert_eq!(block_hash.0[31] as u64, block_number + 1);
    }

    #[test]
    fn test_decode_raw_tx() {
        let temp_dir = TempDir::new().unwrap();
        let db = Brc20ProgDatabase::new(temp_dir.path()).unwrap();
        let engine = BRC20ProgEngine::new(db);

        let raw_tx = hex::decode("f875098504a817c800825208943535353535353535353535353535353535353535880de0b6b3a764000084deadbeef8584a4866483a028ef61340bd939bc2195fe537567866003e1a15d3c71ff63e1590620aa636276a067cbe9d8997f761aecb703304b3800ccf555c9f3dc64214b297fb1966a3b6d83").unwrap();

        let result = engine.get_info_from_raw_tx(raw_tx).unwrap();

        assert_eq!(result.nonce, Some(9));
        assert_eq!(
            result.from.to_string().to_lowercase(),
            "0x5cc06c45e30d85d863766a6f923658f44f8044af"
        );
        assert_eq!(
            result.to.into_to().unwrap().to_string(),
            "0x3535353535353535353535353535353535353535"
        );
        assert_eq!(result.data, vec![0xde, 0xad, 0xbe, 0xef]);
    }
}
