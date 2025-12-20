#![cfg(feature = "server")]

use std::collections::HashMap;
use std::error::Error;
use std::time::{Duration, UNIX_EPOCH};

use alloy::consensus::transaction::RlpEcdsaDecodableTx;
use alloy::consensus::{SignableTransaction, TxLegacy};
use alloy::primitives::{keccak256, Address, B256, U256};
use alloy_rpc_types_trace::geth::CallConfig;
use either::Either::{Left, Right};
use revm::context::ContextTr;
use revm::handler::EvmTr;
use revm::inspector::InspectorEvmTr;
use revm::primitives::Bytes;
use revm::{ExecuteEvm, InspectCommitEvm};
use serde_either::SingleOrVec;

use crate::brc20_controller::{load_brc20_deploy_tx, verify_brc20_contract_address};
use crate::db::types::{BlockResponseED, BytecodeED, LogED, Signature, TraceED, TxED, TxReceiptED};
use crate::db::Brc20ProgDatabase;
use crate::engine::evm::get_evm;
use crate::engine::hardforks::use_rlp_hash_for_tx_hash;
use crate::engine::utils::{
    get_contract_address, get_gas_limit, get_inscription_byte_len, get_tx_hash, LastBlockInfo,
    TxInfo,
};
use crate::engine::validate_bitcoin_rpc_status;
use crate::global::{
    SharedData, CONFIG, MAX_FUTURE_TRANSACTION_BLOCKS, MAX_FUTURE_TRANSACTION_NONCES,
    MAX_REORG_HISTORY_SIZE,
};
use crate::types::{AddressED, PrecompileData};

pub struct BRC20ProgEngine {
    db: SharedData<Brc20ProgDatabase>,
    last_block_info: SharedData<LastBlockInfo>,
}

pub struct ReadContractResult {
    pub status: bool,
    pub status_string: String,
    pub gas_used: u64,
    pub output: Option<Bytes>,
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
                // Check status of Bitcoin RPC
                tracing::info!("Checking Bitcoin RPC status...");
                validate_bitcoin_rpc_status()
                    .map_err(|e| format!("Bitcoin RPC status check failed: {}", e))?;

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
            "BRC20_CONTROLLER_INIT".to_string(),
            u64::MAX,
            [0u8; 32].into(),
        )?;

        let brc20_controller_contract = result
            .contract_address
            .ok_or("Failed to deploy BRC20_Controller")?;

        verify_brc20_contract_address(&brc20_controller_contract.address.to_string())
            .map_err(|_| "Invalid BRC20_Controller contract address")?;

        self.finalise_block(genesis_timestamp, genesis_height, genesis_hash, 1)?;

        // Check status of Bitcoin RPC
        tracing::info!("Checking Bitcoin RPC status...");
        validate_bitcoin_rpc_status()
            .map_err(|e| format!("Bitcoin RPC status check failed: {}", e))?;

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

    pub fn get_all_pending_transactions(
        &self,
    ) -> Result<HashMap<AddressED, HashMap<u64, TxED>>, Box<dyn Error>> {
        let pending_txes = self.db.read().get_all_pending_txes()?;
        let mut result = HashMap::new();

        for ((account, nonce), mut tx) in pending_txes {
            let account_ed: AddressED = account.into();
            let tx_nonce: u64 = nonce.into();

            if !result.contains_key(&account_ed) {
                result.insert(account_ed.clone(), HashMap::new());
            }

            let txes = result.get_mut(&account_ed).unwrap();
            tx.block_hash = B256::ZERO.into();
            tx.block_number = None;
            tx.transaction_index = None;
            txes.insert(tx_nonce, tx);
        }

        Ok(result)
    }

    pub fn get_pending_transactions_from(
        &self,
        account: Address,
    ) -> Result<HashMap<AddressED, HashMap<u64, TxED>>, Box<dyn Error>> {
        let pending_txes = self.db.read().get_all_pending_txes_from(account)?;
        let mut result = HashMap::new();
        result.insert(account.into(), HashMap::new());
        let Some(txes) = result.get_mut(&account.into()) else {
            return Err("Failed to get pending transactions for account".into());
        };

        for ((_, nonce), mut tx) in pending_txes {
            tx.block_hash = B256::ZERO.into();
            tx.block_number = None;
            tx.transaction_index = None;
            txes.insert(nonce.into(), tx);
        }

        Ok(result)
    }

    pub fn get_raw_receipts(
        &self,
        block_number: u64,
    ) -> Result<Option<Vec<String>>, Box<dyn Error>> {
        self.db.read_fn(|db| {
            let Some(raw_block) = db.get_raw_block_by_number(block_number)? else {
                return Ok(None);
            };
            Ok(Some(raw_block.raw_receipts()))
        })
    }

    pub fn get_raw_block(&self, block_number: u64) -> Result<Option<String>, Box<dyn Error>> {
        self.db.read_fn(|db| {
            let Some(raw_block) = db.get_raw_block_by_number(block_number)? else {
                return Ok(None);
            };
            Ok(Some(raw_block.raw_block()))
        })
    }

    pub fn get_raw_header(&self, block_number: u64) -> Result<Option<String>, Box<dyn Error>> {
        self.db.read_fn(|db| {
            let Some(raw_block) = db.get_raw_block_by_number(block_number)? else {
                return Ok(None);
            };
            Ok(Some(raw_block.raw_header()))
        })
    }

    pub fn add_raw_tx_to_block(
        &self,
        timestamp: u64,
        raw_tx: Vec<u8>,
        tx_idx: u64,
        block_number: u64,
        mut block_hash: B256,
        inscription_id: String,
        inscription_byte_len: u64,
        op_return_tx_id: B256,
    ) -> Result<Vec<TxReceiptED>, Box<dyn Error>> {
        // This allows testing, and generating hashes for blocks with unknown hashes
        if block_hash == B256::ZERO {
            block_hash = generate_block_hash(block_number);
        }

        let tx_info =
            self.get_info_from_raw_tx(raw_tx.clone(), use_rlp_hash_for_tx_hash(block_number))?;

        let Some(tx_info) = tx_info else {
            return Ok(Vec::new());
        };

        let gas_limit = get_gas_limit(inscription_byte_len);
        let account_nonce = self.get_account_nonce(tx_info.from)?;

        if let Some(nonce) = tx_info.nonce {
            if nonce != account_nonce {
                if nonce > account_nonce && nonce < account_nonce + MAX_FUTURE_TRANSACTION_NONCES {
                    self.db.write_fn(|db| {
                        db.set_pending_tx(
                            tx_info.from,
                            nonce,
                            TxED::new(
                                get_tx_hash(&tx_info, nonce).into(),
                                nonce.into(),
                                block_hash.into(),
                                block_number.into(),
                                0u64.into(),
                                tx_info.from.into(),
                                tx_info.to.into_to().map(|x| x.into()),
                                gas_limit.into(),
                                tx_info.data.clone().into(),
                                inscription_id,
                                Signature::new(
                                    tx_info.v.into(),
                                    tx_info.r.into(),
                                    tx_info.s.into(),
                                ),
                            ),
                            op_return_tx_id,
                        )
                    })?;
                }
                return Ok(Vec::new());
            }
        }

        let mut receipts = Vec::new();
        let receipt = self.add_tx_to_block(
            timestamp,
            &tx_info,
            tx_idx,
            block_number,
            block_hash,
            inscription_id,
            inscription_byte_len,
            op_return_tx_id,
        )?;
        receipts.push(receipt);

        // Check if next nonce exists in the pending txes and execute it in the same block
        let mut next_nonce = account_nonce + 1;
        let mut next_tx_idx = tx_idx + 1;
        loop {
            // Loop instead of while to avoid holding the database read lock here
            let Some(pending_tx) = self.db.read().get_pending_tx(tx_info.from, next_nonce)? else {
                break;
            };

            let pending_tx_op_return_tx_id = self
                .db
                .read()
                .get_pending_tx_op_return_tx_id(pending_tx.hash.bytes)?;

            if let Some(pending_tx_block_number) = pending_tx.block_number {
                let pending_tx_block_number: u64 = pending_tx_block_number.into();
                if MAX_FUTURE_TRANSACTION_BLOCKS + pending_tx_block_number > block_number {
                    let receipt = self.add_tx_to_block(
                        timestamp,
                        &TxInfo::from_saved_transaction(
                            pending_tx.from.address,
                            pending_tx.to.map(|x| x.address).into(),
                            pending_tx.input.bytes,
                            pending_tx.nonce.into(),
                            pending_tx.hash.bytes,
                            if pending_tx.v.is_zero() { 0 } else { 1 },
                            pending_tx.r.uint,
                            pending_tx.s.uint,
                        ),
                        next_tx_idx,
                        block_number,
                        block_hash,
                        pending_tx.inscription_id.unwrap_or_default(),
                        get_inscription_byte_len(pending_tx.gas.into()).into(),
                        pending_tx_op_return_tx_id.unwrap_or([0u8; 32].into()).bytes,
                    )?;
                    receipts.push(receipt);
                }
            }
            self.db.write_fn(|db| {
                db.remove_pending_tx(pending_tx.from.address, pending_tx.nonce.into())
            })?;
            next_nonce += 1;
            next_tx_idx += 1;
        }

        Ok(receipts)
    }

    pub fn get_info_from_raw_tx(
        &self,
        mut raw_tx: Vec<u8>,
        use_rlp_hash: bool,
    ) -> Result<Option<TxInfo>, Box<dyn Error>> {
        let (decoded_raw_tx, signature) =
            TxLegacy::rlp_decode_with_signature(&mut raw_tx.as_mut_slice().as_ref())
                .map_err(|_| "Failed to decode legacy transaction")?;

        if decoded_raw_tx.chain_id != Some(CONFIG.read().chain_id) {
            return Ok(None);
        }

        let signing_hash = keccak256(decoded_raw_tx.encoded_for_signing());
        let recovered_address = signature.recover_address_from_prehash(&signing_hash)?;

        // Signing hash is not the tx hash (as it can collide), so compute the correct one
        // This was an oversight in earlier versions
        let tx_hash = if use_rlp_hash {
            keccak256(&raw_tx)
        } else {
            signing_hash
        };

        Ok(Some(TxInfo::from_raw_transaction(
            recovered_address,
            decoded_raw_tx,
            tx_hash,
            signature.v() as u8,
            signature.r(),
            signature.s(),
        )))
    }

    pub fn add_tx_to_block(
        &self,
        timestamp: u64,
        tx_info: &TxInfo,
        tx_idx: u64,
        block_number: u64,
        mut block_hash: B256,
        inscription_id: String,
        inscription_byte_len: u64,
        op_return_tx_id: B256,
    ) -> Result<TxReceiptED, Box<dyn Error>> {
        // This allows testing, and generating hashes for blocks with unknown hashes
        if block_hash == B256::ZERO {
            block_hash = generate_block_hash(block_number);
        }

        self.validate_next_tx(tx_idx, block_hash, block_number, timestamp)?;

        let account_nonce = self.get_account_nonce(tx_info.from)?;
        let tx_nonce = tx_info.nonce.unwrap_or(account_nonce);

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
        let gas_limit = get_gas_limit(inscription_byte_len);

        self.db.write_fn(|db| {
            let processing_start_time = self.last_block_info.read().start_time.elapsed();

            let db_moved = core::mem::take(&mut *db);
            let mut evm = get_evm(
                block_number,
                block_hash,
                timestamp,
                db_moved,
                None,
                op_return_tx_id,
                &None,
            );

            evm.ctx().modify_tx(|tx| {
                tx.caller = tx_info.from;
                tx.kind = tx_info.to;
                tx.data = tx_info.data.clone();
                tx.nonce = tx_nonce;
                tx.gas_limit = gas_limit;
            });

            let tx = evm.ctx().tx().clone();
            let output = evm.inspect_tx_commit(tx);

            core::mem::swap(&mut *db, evm.ctx().db_mut());

            let cumulative_gas_used = self
                .last_block_info
                .read()
                .gas_used
                .checked_add(output.as_ref().map(|o| o.gas_used()).unwrap_or(0))
                .unwrap_or(self.last_block_info.read().gas_used);

            let traces: TraceED = evm
                .inspector()
                .geth_builder()
                .geth_call_traces(
                    CallConfig {
                        only_top_call: Some(false),
                        with_log: Some(true),
                    },
                    output.as_ref().map(|o| o.gas_used()).unwrap_or(0),
                )
                .into();

            // If this is a contract creation, store it
            if let Some(created_contract) = traces.get_created_contract() {
                db.set_contract_address_to_inscription_id(
                    created_contract.address,
                    inscription_id.clone(),
                )?;
            }

            if CONFIG.read().evm_record_traces {
                db.set_tx_trace(tx_hash, traces)?;
            }

            db.set_tx_receipt(
                block_hash,
                block_number,
                output
                    .as_ref()
                    .map(|output| get_contract_address(output))
                    .unwrap_or(None),
                tx_info.from,
                tx_info.to_address_optional(),
                &tx_info.data,
                tx_hash,
                tx_idx,
                output.as_ref().ok().map(|output| output.clone()),
                cumulative_gas_used,
                tx_nonce,
                self.last_block_info.read().log_index,
                inscription_id,
                gas_limit,
                tx_info.v.into(),
                tx_info.r.into(),
                tx_info.s.into(),
            )?;

            self.last_block_info.write_fn_unchecked(|last_block_info| {
                last_block_info.waiting_tx_count += 1;
                last_block_info.gas_used = last_block_info
                    .gas_used
                    .checked_add(output.as_ref().map(|o| o.gas_used()).unwrap_or(0))
                    .unwrap_or(last_block_info.gas_used);
                last_block_info.log_index +=
                    output.as_ref().map(|o| o.logs()).unwrap_or(&[]).len() as u64;
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

    pub fn get_block_trace_hash(
        &self,
        block_number: u64,
    ) -> Result<Option<String>, Box<dyn Error>> {
        let Some(trace_hash_str) = self.get_block_trace_string(block_number)? else {
            return Ok(None);
        };
        let digest = sha256::digest(trace_hash_str);
        Ok(Some(digest))
    }

    pub fn get_block_trace_string(
        &self,
        block_number: u64,
    ) -> Result<Option<String>, Box<dyn Error>> {
        static TRACE_SEPARATOR: &str = "|";
        self.db.read_fn(|db| {
            let Some(block) = db.get_block(block_number)? else {
                return Ok(None);
            };
            let Left(transactions) = block.transactions else {
                return Ok(None);
            };
            // Sort by tx index (as they may be out of order)
            let mut transactions = transactions;
            transactions.sort_by_key(|tx_hash| {
                db.get_tx_receipt(tx_hash.bytes)
                    .ok()
                    .flatten()
                    .expect("Transaction in block not found in database")
                    .transaction_index
            });
            let mut trace_hash_str = String::new();
            for tx_hash in transactions {
                if let Some(trace) = db.get_tx_trace(tx_hash.bytes)? {
                    trace_hash_str.push_str(&trace.get_opi_string());
                    trace_hash_str.push_str(TRACE_SEPARATOR);
                }
            }
            Ok(Some(
                trace_hash_str.trim_end_matches(TRACE_SEPARATOR).to_string(),
            ))
        })
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

            // Save the full block info in the database for ease of access
            let block_response = db.generate_block(
                block_hash,
                block_number,
                timestamp,
                gas_used,
                total_time_took,
            )?;
            db.set_block(block_number, block_response.clone())?;
            db.set_raw_block(block_number, db.generate_raw_block(block_response)?)?;

            // Remove old transactions from the txpool
            db.clear_txpool(block_number)?;

            // Set block hash last to avoid race conditions
            db.set_block_hash(block_number, block_hash)
        })?;

        self.last_block_info.write_fn_unchecked(|last_block_info| {
            *last_block_info = LastBlockInfo::new();
        });

        Ok(())
    }

    pub fn read_contract(
        &self,
        tx_info: &TxInfo,
        block_height: Option<u64>,
        gas_limit: Option<u64>,
    ) -> Result<ReadContractResult, Box<dyn Error>> {
        self.require_no_waiting_txes()?;

        let block_number = if let Some(height) = block_height {
            height
        } else {
            self.get_next_block_height()?
        };

        let timestamp = UNIX_EPOCH.elapsed().map(|x| x.as_secs())?;
        let nonce = self.get_account_nonce(tx_info.from)?;

        // This isn't actually writing to the database, but the EVM context requires a mutable reference
        let output = self.db.write_fn(|db| {
            let db_moved = core::mem::take(&mut *db);
            let mut evm = get_evm(
                block_number,
                B256::ZERO,
                timestamp,
                db_moved,
                None,
                [0u8; 32].into(),
                &None,
            );

            evm.ctx().modify_tx(|tx| {
                tx.caller = tx_info.from;
                tx.kind = tx_info.to;
                tx.data = tx_info.data.clone();
                tx.nonce = nonce;
                tx.gas_limit = gas_limit.unwrap_or(CONFIG.read().evm_call_gas_limit);
            });

            let output = evm.replay().map(|x| x.result);
            core::mem::swap(&mut *db, evm.ctx().db_mut());

            output.map_err(|e| e.into())
        })?;

        Ok(ReadContractResult {
            status: output.is_success(),
            status_string: format!("{:?}", output),
            gas_used: output.gas_used().into(),
            output: output.output().cloned(),
        })
    }

    pub fn read_contract_multi(
        &self,
        tx_infos: &Vec<TxInfo>,
        block_height: Option<u64>,
        precompile_data: Option<PrecompileData>,
        gas_limit: Option<&Vec<u64>>,
    ) -> Result<Vec<ReadContractResult>, Box<dyn Error>> {
        self.require_no_waiting_txes()?;

        let block_number = if let Some(height) = block_height {
            height
        } else {
            self.get_next_block_height()?
        };

        let timestamp = UNIX_EPOCH.elapsed().map(|x| x.as_secs())?;
        let mut nonces = HashMap::new();
        for tx_info in tx_infos {
            let nonce = self.get_account_nonce(tx_info.from)?;
            nonces.insert(tx_info.from, nonce);
        }

        // This isn't actually writing to the database, but the EVM context requires a mutable reference
        let outputs = self.db.write_fn(|db| {
            let db_moved = core::mem::take(&mut *db);
            let mut evm = get_evm(
                block_number,
                B256::ZERO,
                timestamp,
                db_moved,
                None,
                [0u8; 32].into(),
                &precompile_data,
            );

            let mut outputs = Vec::new();
            for idx in 0..tx_infos.len() {
                let tx_info = &tx_infos[idx];
                let nonce = *nonces.get(&tx_info.from).unwrap_or(&0);
                nonces.insert(tx_info.from, nonce + 1);

                evm.precompiles.op_return_tx_id = precompile_data
                    .as_ref()
                    .and_then(|data| data.op_return_tx_ids.get(idx).cloned())
                    .unwrap_or([0u8; 32].into())
                    .into();

                evm.ctx().modify_tx(|tx| {
                    tx.caller = tx_info.from;
                    tx.kind = tx_info.to;
                    tx.data = tx_info.data.clone();
                    tx.nonce = nonce;
                    tx.gas_limit = gas_limit.map_or(CONFIG.read().evm_call_gas_limit, |gl| gl.get(idx).cloned().unwrap_or(CONFIG.read().evm_call_gas_limit));
                });
                let tx = evm.ctx().tx().clone();
                outputs.push(evm.transact_one(tx)?);
            }

            core::mem::swap(&mut *db, evm.ctx().db_mut());

            Ok(outputs)
        })?;

        let mut results = Vec::new();
        for output in outputs {
            results.push(ReadContractResult {
                status: output.is_success(),
                status_string: format!("{:?}", output),
                gas_used: output.gas_used().into(),
                output: output.output().cloned(),
            });
        }

        Ok(results)
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
                inscription_id.clone(),
                1000,
                [0u8; 32].into(),
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
                "test_inscription_id".to_string(),
                1000,
                [0u8; 32].into(),
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
                "test_inscription_id".to_string(),
                1000,
                [0u8; 32].into(),
            )
            .unwrap();

        engine
            .add_tx_to_block(
                1622547800,
                &TxInfo::from_inscription(account_2, TxKind::Create, vec![].into()),
                1,
                0,
                B256::ZERO,
                "test_inscription_id".to_string(),
                1000,
                [0u8; 32].into(),
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
                "test_inscription_id".to_string(),
                1000,
                [0u8; 32].into(),
            )
            .unwrap();

        engine
            .add_tx_to_block(
                1622547800,
                &TxInfo::from_inscription(*INDEXER_ADDRESS, TxKind::Create, vec![].into()),
                1,
                0,
                hash,
                "test_inscription_id_2".to_string(),
                1000,
                [10u8; 32].into(),
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
                "test_inscription_id".to_string(),
                1000,
                [0u8; 32].into(),
            )
            .unwrap();

        engine
            .add_tx_to_block(
                1622547800,
                &TxInfo::from_inscription(account_2, TxKind::Create, vec![].into()),
                1,
                0,
                hash,
                "test_inscription_id_2".to_string(),
                1000,
                [0u8; 32].into(),
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
                "test_inscription_id".to_string(),
                1000,
                [0u8; 32].into(),
            )
            .unwrap();

        engine
            .add_tx_to_block(
                1622547800,
                &TxInfo::from_inscription(account_2, TxKind::Create, vec![].into()),
                1,
                0,
                hash,
                "test_inscription_id_2".to_string(),
                1000,
                [10u8; 32].into(),
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
                "test_inscription_id".to_string(),
                1000,
                [0u8; 32].into(),
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
                inscription_id.clone(),
                1000,
                [0u8; 32].into(),
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
                "test_inscription_id".to_string(),
                1000,
                [0u8; 32].into(),
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
                "test_inscription_id".to_string(),
                10000,
                [0u8; 32].into(),
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
                "test_inscription_id".to_string(),
                1000,
                [0u8; 32].into(),
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
                "test_inscription_id_2".to_string(),
                1000,
                [0u8; 32].into(),
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
                "test_inscription_id".to_string(),
                1000,
                [0u8; 32].into(),
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
        CONFIG.write_fn_unchecked(|config| {
            config.chain_id = 0x4252433230;
        });

        let raw_tx = hex::decode("f875098504a817c800825208943535353535353535353535353535353535353535880de0b6b3a764000084deadbeef8584a4866483a028ef61340bd939bc2195fe537567866003e1a15d3c71ff63e1590620aa636276a067cbe9d8997f761aecb703304b3800ccf555c9f3dc64214b297fb1966a3b6d83").unwrap();

        let result = engine.get_info_from_raw_tx(raw_tx, true).unwrap().unwrap();

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

    #[test]
    fn test_decode_raw_tx_new_tx_hash() {
        let temp_dir = TempDir::new().unwrap();
        let db = Brc20ProgDatabase::new(temp_dir.path()).unwrap();
        let engine = BRC20ProgEngine::new(db);
        CONFIG.write_fn_unchecked(|config| {
            config.chain_id = 0x4252433230;
        });

        let raw_tx = hex::decode("f875098504a817c800825208943535353535353535353535353535353535353535880de0b6b3a764000084deadbeef8584a4866483a028ef61340bd939bc2195fe537567866003e1a15d3c71ff63e1590620aa636276a067cbe9d8997f761aecb703304b3800ccf555c9f3dc64214b297fb1966a3b6d83").unwrap();

        let result = engine.get_info_from_raw_tx(raw_tx, true).unwrap().unwrap();

        assert_eq!(
            hex::encode(result.pre_hash.unwrap().0),
            "a868b98e61fc95116dd30d095930cb88eacdd2129c7596744cb36e645952ebfb"
        );
    }

    #[test]
    fn test_decode_raw_tx_old_tx_hash() {
        let temp_dir = TempDir::new().unwrap();
        let db = Brc20ProgDatabase::new(temp_dir.path()).unwrap();
        let engine = BRC20ProgEngine::new(db);
        CONFIG.write_fn_unchecked(|config| {
            config.chain_id = 0x4252433230;
        });

        let raw_tx = hex::decode("f875098504a817c800825208943535353535353535353535353535353535353535880de0b6b3a764000084deadbeef8584a4866483a028ef61340bd939bc2195fe537567866003e1a15d3c71ff63e1590620aa636276a067cbe9d8997f761aecb703304b3800ccf555c9f3dc64214b297fb1966a3b6d83").unwrap();

        let result = engine.get_info_from_raw_tx(raw_tx, false).unwrap().unwrap();

        assert_eq!(
            hex::encode(result.pre_hash.unwrap().0),
            "e591d2cd4e58a3fa496324f525942ff9a03a4b07a77d9d6b31ca14728331aeee"
        );
    }
}
