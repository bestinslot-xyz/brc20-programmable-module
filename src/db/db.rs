use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::path::Path;

use alloy_primitives::map::foldhash::fast::RandomState;
use alloy_primitives::{Address, Bloom, Bytes, FixedBytes, Log, B256, U128, U256, U64};
use revm::context::result::ExecutionResult;
use revm::context::DBErrorMarker;
use revm::{Database as DatabaseTrait, DatabaseCommit};
use revm_state::{Account, AccountInfo, Bytecode};
use rs_merkle::algorithms::Sha256;
use rs_merkle::MerkleTree;
use serde_either::SingleOrVec;

use crate::db::cached_database::{BlockCachedDatabase, BlockHistoryCacheData};
use crate::db::database::BlockDatabase;
use crate::db::types::{
    AccountInfoED, AddressED, BlockResponseED, BytecodeED, LogED, TraceED, TxED, TxReceiptED,
    B256ED, U128ED, U256ED, U512ED, U64ED,
};
use crate::global::{GAS_PER_BYTE, MAX_BLOCK_SIZE};

const MAX_HISTORY_SIZE: u64 = 10;

pub struct DB {
    /// Account address to memory location
    /// TODO: If the value is zero, consider deleting it from the database to save space
    db_account_memory: Option<BlockCachedDatabase<U512ED, U256ED, BlockHistoryCacheData<U256ED>>>,

    /// Code hash to bytecode
    db_code: Option<BlockCachedDatabase<B256ED, BytecodeED, BlockHistoryCacheData<BytecodeED>>>,

    /// Account address to account info
    db_account:
        Option<BlockCachedDatabase<AddressED, AccountInfoED, BlockHistoryCacheData<AccountInfoED>>>,

    /// Block number and index to tx hash
    db_number_and_index_to_tx_hash:
        Option<BlockCachedDatabase<U128ED, B256ED, BlockHistoryCacheData<B256ED>>>,

    /// TxHash to tx receipt
    db_tx_receipt:
        Option<BlockCachedDatabase<B256ED, TxReceiptED, BlockHistoryCacheData<TxReceiptED>>>,

    /// Tx hash to Tx
    db_tx: Option<BlockCachedDatabase<B256ED, TxED, BlockHistoryCacheData<TxED>>>,

    /// TxHash to trace
    db_tx_trace: Option<BlockCachedDatabase<B256ED, TraceED, BlockHistoryCacheData<TraceED>>>,

    /// Hash of Inscription IDs to TxHash
    db_inscription_id_to_tx_hash:
        Option<BlockCachedDatabase<String, B256ED, BlockHistoryCacheData<B256ED>>>,

    /// Contract address to inscription ID
    db_contract_address_to_inscription_id:
        Option<BlockCachedDatabase<AddressED, String, BlockHistoryCacheData<String>>>,

    /// Block hash to block number
    db_block_hash_to_number:
        Option<BlockCachedDatabase<B256ED, U64ED, BlockHistoryCacheData<U64ED>>>,

    // Block number to Block
    db_block_number_to_block: Option<BlockDatabase<BlockResponseED>>,

    /// Block number to block hash
    db_block_number_to_hash: Option<BlockDatabase<B256ED>>,

    /// Block number to block timestamp
    db_block_number_to_timestamp: Option<BlockDatabase<U64ED>>,

    /// Block number to gas used
    db_block_number_to_gas_used: Option<BlockDatabase<U64ED>>,

    /// Block number to mine timestamp
    db_block_number_to_mine_tm: Option<BlockDatabase<U128ED>>,

    /// Cache for latest block number and block hash
    latest_block_number: Option<(u64, B256)>,
}

impl Default for DB {
    fn default() -> Self {
        Self {
            db_account_memory: None,
            db_code: None,
            db_account: None,
            db_number_and_index_to_tx_hash: None,
            db_tx_receipt: None,
            db_tx: None,
            db_tx_trace: None,
            db_inscription_id_to_tx_hash: None,
            db_contract_address_to_inscription_id: None,
            db_block_number_to_block: None,
            db_block_number_to_hash: None,
            db_block_hash_to_number: None,
            db_block_number_to_timestamp: None,
            db_block_number_to_gas_used: None,
            db_block_number_to_mine_tm: None,
            latest_block_number: None,
        }
    }
}

impl DB {
    pub fn new(base_path: &Path) -> Result<Self, Box<dyn Error>> {
        rlimit::Resource::NOFILE.set(4096, 8192)?;

        Ok(Self {
            db_account_memory: Some(BlockCachedDatabase::new(&base_path, "account_memory")?),
            db_code: Some(BlockCachedDatabase::new(&base_path, "code")?),
            db_account: Some(BlockCachedDatabase::new(&base_path, "account")?),
            db_number_and_index_to_tx_hash: Some(BlockCachedDatabase::new(
                &base_path,
                "number_and_index_to_tx_hash",
            )?),
            db_tx_receipt: Some(BlockCachedDatabase::new(&base_path, "tx_receipt")?),
            db_inscription_id_to_tx_hash: Some(BlockCachedDatabase::new(
                &base_path,
                "inscription_id_to_tx_hash",
            )?),
            db_contract_address_to_inscription_id: Some(BlockCachedDatabase::new(
                &base_path,
                "contract_address_to_inscription_id",
            )?),
            db_tx: Some(BlockCachedDatabase::new(&base_path, "tx")?),
            db_tx_trace: Some(BlockCachedDatabase::new(&base_path, "tx_trace")?),
            db_block_hash_to_number: Some(BlockCachedDatabase::new(
                &base_path,
                "block_hash_to_number",
            )?),
            db_block_number_to_block: Some(BlockDatabase::new(
                &base_path,
                "block_number_to_block",
            )?),
            db_block_number_to_hash: Some(BlockDatabase::new(&base_path, "block_number_to_hash")?),
            db_block_number_to_timestamp: Some(BlockDatabase::new(
                &base_path,
                "block_number_to_timestamp",
            )?),
            db_block_number_to_gas_used: Some(BlockDatabase::new(
                &base_path,
                "block_number_to_gas_used",
            )?),
            db_block_number_to_mine_tm: Some(BlockDatabase::new(
                &base_path,
                "block_number_to_mine_tm",
            )?),
            latest_block_number: None,
        })
    }

    pub fn get_latest_block_height(&self) -> Result<u64, Box<dyn Error>> {
        match self.latest_block_number {
            Some((block_number, _)) => return Ok(block_number),
            None => {
                return Ok(self
                    .db_block_number_to_hash
                    .as_ref()
                    .ok_or("DB Error")?
                    .last_key()?
                    .unwrap_or(0));
            }
        }
    }

    pub fn get_account_memory(
        &self,
        account: Address,
        mem_loc: U256,
    ) -> Result<Option<U256ED>, Box<dyn Error>> {
        self.db_account_memory
            .as_ref()
            .ok_or("DB Error")?
            .latest(&U512ED::from_addr_u256(account, mem_loc)?)
    }

    pub fn set_account_memory(
        &mut self,
        account: Address,
        mem_loc: U256,
        value: U256,
    ) -> Result<(), Box<dyn Error>> {
        let block_number = self.get_latest_block_height()?;
        self.db_account_memory.as_mut().ok_or("DB Error")?.set(
            block_number,
            &U512ED::from_addr_u256(account, mem_loc)?,
            value.into(),
        )?;

        Ok(())
    }

    pub fn get_code(&self, code_hash: B256) -> Result<Option<BytecodeED>, Box<dyn Error>> {
        self.db_code
            .as_ref()
            .ok_or("DB Error")?
            .latest(&code_hash.into())
    }

    pub fn set_code(&mut self, code_hash: B256, bytecode: Bytecode) -> Result<(), Box<dyn Error>> {
        let block_number = self.get_latest_block_height()?;
        self.db_code.as_mut().ok_or("DB Error")?.set(
            block_number,
            &code_hash.into(),
            bytecode.into(),
        )
    }

    fn get_number_and_index_key(block_number: u64, tx_idx: u64) -> u128 {
        ((block_number as u128) << 64) | tx_idx as u128
    }

    pub fn get_logs(
        &self,
        block_number_from: Option<u64>,
        block_number_to: Option<u64>,
        contract_address: Option<Address>,
        topics: Option<Vec<SingleOrVec<Option<B256>>>>,
    ) -> Result<Vec<LogED>, Box<dyn Error>> {
        let latest_block_number = self.get_latest_block_height()?;
        let block_number_from = block_number_from.unwrap_or(latest_block_number);
        let block_number_to = block_number_to.unwrap_or(block_number_from);

        // Limit the number of blocks to be fetched
        // DB is under a lock here and if this takes too long, it will block other threads
        //
        // TODO: This is a temporary solution, we can potentially avoid using a mutex for reads
        // TODO: Also, test this, maybe it's not that slow?
        if block_number_to - block_number_from > 5 {
            return Err("Block range is too large, please limit it to 5 blocks".into());
        }

        let mut logs = Vec::new();

        let tx_ids = self
            .db_number_and_index_to_tx_hash
            .as_ref()
            .ok_or("DB Error")?
            .get_range(
                &Self::get_number_and_index_key(block_number_from, 0).into(),
                &Self::get_number_and_index_key(block_number_to + 1, 0).into(),
            )?;

        for tx_pair in tx_ids {
            let tx_id = tx_pair.1;
            let Some(tx_receipt) = self.get_tx_receipt(tx_id.into())? else {
                continue;
            };

            for log in tx_receipt.logs {
                if let Some(ref address) = contract_address {
                    if log.address.address != *address {
                        continue;
                    }
                }

                let mut matched = true;
                if let Some(ref topics) = topics {
                    for idx in 0..topics.len() {
                        match topics[idx] {
                            SingleOrVec::Single(ref topic) => {
                                if let Some(ref topic) = topic {
                                    if log.topics.len() <= idx || log.topics[idx].bytes != *topic {
                                        matched = false;
                                        break;
                                    }
                                }
                            }
                            SingleOrVec::Vec(ref topics) => {
                                if log.topics.len() <= idx {
                                    matched = false;
                                    break;
                                }
                                if !topics.iter().any(|x| {
                                    if let Some(ref topic) = x {
                                        log.topics[idx].bytes == *topic
                                    } else {
                                        // We ignore the None case in OR condition, it's an invalid case
                                        // and we don't want to match all
                                        false
                                    }
                                }) {
                                    matched = false;
                                    break;
                                }
                            }
                        }
                    }
                }

                if matched {
                    logs.push(log);
                }
            }
        }

        Ok(logs)
    }

    pub fn get_tx_count(
        &self,
        account: Option<Address>,
        block_number: u64,
    ) -> Result<u64, Box<dyn Error>> {
        let tx_ids = self
            .db_number_and_index_to_tx_hash
            .as_ref()
            .ok_or("DB Error")?
            .get_range(
                &Self::get_number_and_index_key(block_number, 0).into(),
                &Self::get_number_and_index_key(block_number + 1, 0).into(),
            )?;

        let mut count = 0;
        for tx_pair in tx_ids {
            let tx_id = tx_pair.1;
            let tx = self.get_tx_by_hash(tx_id.into())?;
            if account.is_none() || tx.map(|tx| tx.from.address) == account {
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn get_tx_hash_by_inscription_id(
        &self,
        inscription_id: String,
    ) -> Result<Option<B256ED>, Box<dyn Error>> {
        self.db_inscription_id_to_tx_hash
            .as_ref()
            .ok_or("DB Error")?
            .latest(&inscription_id)
    }

    pub fn get_inscription_id_by_contract_address(
        &self,
        contract_address: Address,
    ) -> Result<Option<String>, Box<dyn Error>> {
        self.db_contract_address_to_inscription_id
            .as_ref()
            .ok_or("DB Error")?
            .latest(&contract_address.into())
    }

    pub fn set_contract_address_to_inscription_id(
        &mut self,
        contract_address: Address,
        inscription_id: String,
    ) -> Result<(), Box<dyn Error>> {
        let block_number = self.get_latest_block_height()?;
        Ok(self
            .db_contract_address_to_inscription_id
            .as_mut()
            .ok_or("DB Error")?
            .set(block_number, &contract_address.into(), inscription_id)?)
    }

    pub fn set_tx_hash_by_inscription_id(
        &mut self,
        inscription_id: String,
        tx_hash: B256,
    ) -> Result<(), Box<dyn Error>> {
        let block_number = self.get_latest_block_height()?;
        Ok(self
            .db_inscription_id_to_tx_hash
            .as_mut()
            .ok_or("DB Error")?
            .set(block_number, &inscription_id, tx_hash.into())?)
    }

    pub fn get_tx_hash_by_block_number_and_index(
        &self,
        block_number: u64,
        tx_idx: u64,
    ) -> Result<Option<B256ED>, Box<dyn Error>> {
        self.db_number_and_index_to_tx_hash
            .as_ref()
            .ok_or("DB Error")?
            .latest(&Self::get_number_and_index_key(block_number, tx_idx).into())
    }

    pub fn get_tx_hash_by_block_hash_and_index(
        &self,
        block_hash: B256,
        tx_idx: u64,
    ) -> Result<Option<B256ED>, Box<dyn Error>> {
        return if let Some(block_number) = self.get_block_number(block_hash)? {
            self.get_tx_hash_by_block_number_and_index(block_number.into(), tx_idx)
        } else {
            return Ok(None);
        };
    }

    pub fn get_tx_by_hash(&self, tx_hash: B256) -> Result<Option<TxED>, Box<dyn Error>> {
        self.db_tx
            .as_ref()
            .ok_or("DB Error")?
            .latest(&tx_hash.into())
    }

    pub fn get_tx_receipt(&self, tx_hash: B256) -> Result<Option<TxReceiptED>, Box<dyn Error>> {
        self.db_tx_receipt
            .as_ref()
            .ok_or("DB Error")?
            .latest(&tx_hash.into())
    }

    pub fn require_block_does_not_exist(
        &self,
        block_hash: B256,
        block_number: u64,
    ) -> Result<(), Box<dyn Error>> {
        if self.get_block_hash(block_number)?.is_some() {
            return Err(format!("Block with number {} already exists", block_number).into());
        }
        if self.get_block_number(block_hash)?.is_some() {
            return Err(format!("Block with hash {} already exists", block_hash).into());
        }
        Ok(())
    }

    pub fn get_tx_trace(&self, tx_hash: B256) -> Result<Option<TraceED>, Box<dyn Error>> {
        self.db_tx_trace
            .as_ref()
            .ok_or("DB Error")?
            .latest(&tx_hash.into())
    }

    pub fn set_tx_trace(&mut self, tx_hash: B256, trace: TraceED) -> Result<(), Box<dyn Error>> {
        self.db_tx_trace
            .as_mut()
            .ok_or("DB Error")?
            .set(0, &tx_hash.into(), trace)
    }

    pub fn set_tx_receipt(
        &mut self,
        result_type: &str,
        reason: &str,
        result: Option<&Bytes>,
        block_hash: B256,
        block_number: u64,
        block_timestamp: u64,
        contract_address: Option<Address>,
        from: Address,
        to: Option<Address>,
        data: &Bytes,
        tx_hash: B256,
        tx_idx: u64,
        output: &ExecutionResult,
        cumulative_gas_used: u64,
        nonce: u64,
        start_log_index: u64,
        inscription_id: Option<String>,
        gas_limit: u64,
    ) -> Result<(), Box<dyn Error>> {
        self.require_block_does_not_exist(block_hash, block_number)?;

        let tx_receipt = TxReceiptED::new(
            block_hash.into(),
            block_number.into(),
            block_timestamp.into(),
            contract_address.map(AddressED::new),
            from.into(),
            to.map(AddressED::new),
            tx_hash.into(),
            tx_idx.into(),
            output,
            cumulative_gas_used.into(),
            nonce.into(),
            start_log_index.into(),
            result_type.to_string(),
            reason.to_string(),
            result.map(|x| x.clone().into()),
        )?;

        let tx = TxED::new(
            tx_hash.into(),
            nonce.into(),
            block_hash.into(),
            block_number.into(),
            tx_idx.into(),
            from.into(),
            to.map(AddressED::new),
            gas_limit.into(),
            data.clone().into(),
            inscription_id.clone(),
        );

        self.db_tx
            .as_mut()
            .ok_or("DB Error")?
            .set(block_number, &tx_hash.into(), tx)?;

        self.db_number_and_index_to_tx_hash
            .as_mut()
            .ok_or("DB Error")?
            .set(
                block_number,
                &Self::get_number_and_index_key(block_number, tx_idx).into(),
                tx_hash.into(),
            )?;

        if let Some(inscription_id) = inscription_id {
            self.set_tx_hash_by_inscription_id(inscription_id, tx_hash)?;
        }

        Ok(self.db_tx_receipt.as_mut().ok_or("DB Error")?.set(
            block_number,
            &tx_hash.into(),
            tx_receipt,
        )?)
    }

    pub fn get_account_info(
        &self,
        account: Address,
    ) -> Result<Option<AccountInfoED>, Box<dyn Error>> {
        self.db_account
            .as_ref()
            .ok_or("DB Error")?
            .latest(&account.into())
    }

    pub fn set_account_info(
        &mut self,
        account: Address,
        value: AccountInfo,
    ) -> Result<(), Box<dyn Error>> {
        let block_number = self.get_latest_block_height()?;
        Ok(self.db_account.as_mut().ok_or("DB Error")?.set(
            block_number,
            &account.into(),
            value.into(),
        )?)
    }

    pub fn generate_block(&self, block_number: u64) -> Result<BlockResponseED, Box<dyn Error>> {
        let block_timestamp = self
            .get_block_timestamp(block_number)?
            .ok_or("Block timestamp not set")?;
        let gas_used = self
            .get_gas_used(block_number)?
            .ok_or("Block gasUsed is not set")?;
        let mine_timestamp = self
            .get_mine_timestamp(block_number)?
            .ok_or("Block mine timestamp not set")?;
        let block_hash = self
            .get_block_hash(block_number)?
            .ok_or("Block hash not set")?;

        let parent_hash = if block_number == 0 {
            B256::ZERO
        } else {
            self.get_block_hash(block_number - 1)?
                .ok_or("Parent block is missing")?
        };

        let tx_ids = self
            .db_number_and_index_to_tx_hash
            .as_ref()
            .ok_or("DB Error")?
            .get_range(
                &Self::get_number_and_index_key(block_number, 0).into(),
                &Self::get_number_and_index_key(block_number + 1, 0).into(),
            )?;

        let leaves = tx_ids
            .iter()
            .map(|x| x.1.bytes.0)
            .collect::<Vec<[u8; 32]>>();

        let tx_merkle = MerkleTree::<Sha256>::from_leaves(leaves.as_slice());

        let mut transactions: Vec<B256ED> = Vec::new();
        let mut bloom = Bloom::new([0u8; 256]);
        for tx_pair in tx_ids {
            let tx_id = tx_pair.1.into();
            if let Some(tx_receipt) = self.get_tx_receipt(tx_id)? {
                for log in tx_receipt.logs {
                    bloom.accrue_log(
                        &Log::new(
                            log.address.address,
                            log.topics.iter().map(|x| x.bytes).collect(),
                            log.data.bytes,
                        )
                        .unwrap_or(Log::empty()),
                    );
                }
            }
            transactions.push(tx_id.into());
        }

        let block_response = BlockResponseED::new(
            0u64.into(),
            (*MAX_BLOCK_SIZE * *GAS_PER_BYTE).into(),
            gas_used.as_limbs()[0].into(),
            block_hash.into(),
            FixedBytes(bloom.as_slice().try_into()?).into(),
            (transactions.len() as u64).into(),
            block_number.into(),
            block_timestamp.as_limbs()[0].into(),
            mine_timestamp.into(),
            transactions,
            tx_merkle.root().unwrap_or([0; 32]).into(),
            0u64.into(),
            parent_hash.into(),
            [0; 32].into(),
            0u64.into(),
        );

        Ok(block_response)
    }

    pub fn get_block(&self, block_number: u64) -> Result<Option<BlockResponseED>, Box<dyn Error>> {
        self.db_block_number_to_block
            .as_ref()
            .ok_or("DB Error")?
            .get(block_number)
    }

    pub fn set_block(
        &mut self,
        block_number: u64,
        block_response: BlockResponseED,
    ) -> Result<(), Box<dyn Error>> {
        Ok(self
            .db_block_number_to_block
            .as_mut()
            .ok_or("DB Error")?
            .set(block_number, block_response))
    }

    pub fn get_block_number(&self, block_hash: B256) -> Result<Option<U64ED>, Box<dyn Error>> {
        self.db_block_hash_to_number
            .as_ref()
            .ok_or("DB Error")?
            .latest(&block_hash.into())
    }

    pub fn get_block_hash(&self, block_number: u64) -> Result<Option<B256>, Box<dyn Error>> {
        self.db_block_number_to_hash
            .as_ref()
            .ok_or("DB Error")?
            .get(block_number)
            .map(|op| op.map(|x| x.into()))
    }

    pub fn set_block_hash(
        &mut self,
        block_number: u64,
        block_hash: B256,
    ) -> Result<(), Box<dyn Error>> {
        self.require_block_does_not_exist(block_hash, block_number)?;

        if self.latest_block_number.is_none()
            || block_number > self.latest_block_number.unwrap_or((0, B256::ZERO)).0
        {
            self.latest_block_number = Some((block_number, block_hash));
        }

        self.db_block_number_to_hash
            .as_mut()
            .ok_or("DB Error")?
            .set(block_number, block_hash.into());

        Ok(self
            .db_block_hash_to_number
            .as_mut()
            .ok_or("DB Error")?
            .set(block_number, &block_hash.into(), block_number.into())?)
    }

    pub fn get_block_timestamp(&self, number: u64) -> Result<Option<U64>, Box<dyn Error>> {
        self.db_block_number_to_timestamp
            .as_ref()
            .ok_or("DB Error")?
            .get(number)
            .map(|x| x.map(|x| x.uint))
    }

    pub fn set_block_timestamp(
        &mut self,
        block_number: u64,
        block_timestamp: u64,
    ) -> Result<(), Box<dyn Error>> {
        Ok(self
            .db_block_number_to_timestamp
            .as_mut()
            .ok_or("DB Error")?
            .set(block_number, block_timestamp.into()))
    }

    pub fn get_gas_used(&self, block_number: u64) -> Result<Option<U64>, Box<dyn Error>> {
        self.db_block_number_to_gas_used
            .as_ref()
            .ok_or("DB Error")?
            .get(block_number)
            .map(|x| x.map(|x| x.uint))
    }

    pub fn set_gas_used(&mut self, block_number: u64, gas_used: u64) -> Result<(), Box<dyn Error>> {
        Ok(self
            .db_block_number_to_gas_used
            .as_mut()
            .ok_or("DB Error")?
            .set(block_number, gas_used.into()))
    }

    pub fn get_mine_timestamp(&self, block_number: u64) -> Result<Option<U128>, Box<dyn Error>> {
        self.db_block_number_to_mine_tm
            .as_ref()
            .ok_or("DB Error")?
            .get(block_number)
            .map(|x| x.map(|x| x.uint))
    }

    pub fn set_mine_timestamp(
        &mut self,
        block_number: u64,
        mine_timestamp: u128,
    ) -> Result<(), Box<dyn Error>> {
        self.db_block_number_to_mine_tm
            .as_mut()
            .ok_or("DB Error")?
            .set(block_number, mine_timestamp.into());

        Ok(())
    }

    pub fn commit_changes(&mut self) -> Result<(), Box<dyn Error>> {
        let latest_block_number = self.get_latest_block_height()?;

        self.db_block_number_to_hash
            .as_mut()
            .ok_or("DB Error")?
            .commit()?;
        self.db_block_number_to_timestamp
            .as_mut()
            .ok_or("DB Error")?
            .commit()?;
        self.db_block_number_to_gas_used
            .as_mut()
            .ok_or("DB Error")?
            .commit()?;
        self.db_block_number_to_mine_tm
            .as_mut()
            .ok_or("DB Error")?
            .commit()?;
        self.db_block_number_to_block
            .as_mut()
            .ok_or("DB Error")?
            .commit()?;

        self.db_number_and_index_to_tx_hash
            .as_mut()
            .ok_or("DB Error")?
            .commit(latest_block_number)?;
        self.db_inscription_id_to_tx_hash
            .as_mut()
            .ok_or("DB Error")?
            .commit(latest_block_number)?;
        self.db_contract_address_to_inscription_id
            .as_mut()
            .ok_or("DB Error")?
            .commit(latest_block_number)?;
        self.db_tx
            .as_mut()
            .ok_or("DB Error")?
            .commit(latest_block_number)?;
        self.db_tx_trace
            .as_mut()
            .ok_or("DB Error")?
            .commit(latest_block_number)?;
        self.db_tx_receipt
            .as_mut()
            .ok_or("DB Error")?
            .commit(latest_block_number)?;
        self.db_account_memory
            .as_mut()
            .ok_or("DB Error")?
            .commit(latest_block_number)?;
        self.db_code
            .as_mut()
            .ok_or("DB Error")?
            .commit(latest_block_number)?;
        self.db_account
            .as_mut()
            .ok_or("DB Error")?
            .commit(latest_block_number)?;
        self.db_block_hash_to_number
            .as_mut()
            .ok_or("DB Error")?
            .commit(latest_block_number)?;

        self.clear_caches()?;
        Ok(())
    }

    pub fn clear_caches(&mut self) -> Result<(), Box<dyn Error>> {
        self.db_account_memory
            .as_mut()
            .ok_or("DB Error")?
            .clear_cache();
        self.db_code.as_mut().ok_or("DB Error")?.clear_cache();
        self.db_account.as_mut().ok_or("DB Error")?.clear_cache();
        self.db_block_number_to_hash
            .as_mut()
            .ok_or("DB Error")?
            .clear_cache();
        self.db_block_hash_to_number
            .as_mut()
            .ok_or("DB Error")?
            .clear_cache();
        self.db_inscription_id_to_tx_hash
            .as_mut()
            .ok_or("DB Error")?
            .clear_cache();
        self.db_contract_address_to_inscription_id
            .as_mut()
            .ok_or("DB Error")?
            .clear_cache();
        self.db_tx.as_mut().ok_or("DB Error")?.clear_cache();
        self.db_tx_trace.as_mut().ok_or("DB Error")?.clear_cache();
        self.db_tx_receipt.as_mut().ok_or("DB Error")?.clear_cache();
        self.db_number_and_index_to_tx_hash
            .as_mut()
            .ok_or("DB Error")?
            .clear_cache();
        self.db_block_number_to_timestamp
            .as_mut()
            .ok_or("DB Error")?
            .clear_cache();
        self.db_block_number_to_gas_used
            .as_mut()
            .ok_or("DB Error")?
            .clear_cache();
        self.db_block_number_to_mine_tm
            .as_mut()
            .ok_or("DB Error")?
            .clear_cache();
        self.db_block_number_to_block
            .as_mut()
            .ok_or("DB Error")?
            .clear_cache();

        self.latest_block_number = None;
        Ok(())
    }

    pub fn reorg(&mut self, latest_valid_block_number: u64) -> Result<(), Box<dyn Error>> {
        if self.get_latest_block_height()? - latest_valid_block_number > MAX_HISTORY_SIZE {
            return Err("Latest valid block number is too far behind current block height".into());
        }

        self.db_account_memory
            .as_mut()
            .ok_or("DB Error")?
            .reorg(latest_valid_block_number)?;
        self.db_code
            .as_mut()
            .ok_or("DB Error")?
            .reorg(latest_valid_block_number)?;
        self.db_account
            .as_mut()
            .ok_or("DB Error")?
            .reorg(latest_valid_block_number)?;
        self.db_block_hash_to_number
            .as_mut()
            .ok_or("DB Error")?
            .reorg(latest_valid_block_number)?;
        self.db_number_and_index_to_tx_hash
            .as_mut()
            .ok_or("DB Error")?
            .reorg(latest_valid_block_number)?;
        self.db_tx_receipt
            .as_mut()
            .ok_or("DB Error")?
            .reorg(latest_valid_block_number)?;
        self.db_inscription_id_to_tx_hash
            .as_mut()
            .ok_or("DB Error")?
            .reorg(latest_valid_block_number)?;
        self.db_contract_address_to_inscription_id
            .as_mut()
            .ok_or("DB Error")?
            .reorg(latest_valid_block_number)?;
        self.db_tx
            .as_mut()
            .ok_or("DB Error")?
            .reorg(latest_valid_block_number)?;
        self.db_tx_trace
            .as_mut()
            .ok_or("DB Error")?
            .reorg(latest_valid_block_number)?;

        self.db_block_number_to_hash
            .as_mut()
            .ok_or("DB Error")?
            .reorg(latest_valid_block_number)?;
        self.db_block_number_to_timestamp
            .as_mut()
            .ok_or("DB Error")?
            .reorg(latest_valid_block_number)?;
        self.db_block_number_to_gas_used
            .as_mut()
            .ok_or("DB Error")?
            .reorg(latest_valid_block_number)?;
        self.db_block_number_to_mine_tm
            .as_mut()
            .ok_or("DB Error")?
            .reorg(latest_valid_block_number)?;
        self.db_block_number_to_block
            .as_mut()
            .ok_or("DB Error")?
            .reorg(latest_valid_block_number)?;

        Ok(self.commit_changes()?)
    }
}

#[derive(Debug)]
pub struct DBError(Box<dyn Error>);

impl DBErrorMarker for DBError {}

impl Display for DBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DBError: {}", self.0)
    }
}

impl Error for DBError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.0)
    }
}

impl DatabaseTrait for DB {
    type Error = DBError;

    /// Get basic account information.
    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        self.get_account_info(address)
            .map(|x| {
                x.map(|x| {
                    let mut account_info: AccountInfo = x.into();
                    account_info.code = Some(
                        self.code_by_hash(account_info.code_hash)
                            .unwrap_or(Bytecode::new()),
                    );
                    account_info
                })
            })
            .map_err(|x| DBError(x))
    }

    /// Get account code by its hash.
    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.get_code(code_hash)
            .map(|x| x.map(|x| x.bytecode).unwrap_or(Bytecode::new()))
            .map_err(|x| DBError(x))
    }

    /// Get storage value of address at index.
    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        self.get_account_memory(address, index)
            .map(|x| x.map(|x| x.uint).unwrap_or(U256::ZERO))
            .map_err(|x| DBError(x))
    }

    /// Get block hash by block number.
    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        self.get_block_hash(number)
            .map(|x| x.unwrap_or(B256::ZERO))
            .map_err(|x| DBError(x))
    }
}

impl DatabaseCommit for DB {
    fn commit(&mut self, changes: HashMap<Address, Account, RandomState>) {
        for (address, account) in changes {
            if !account.is_touched() {
                continue;
            }
            let mut acc_info = AccountInfo::default();
            acc_info.balance = account.info.balance;
            acc_info.nonce = account.info.nonce;
            acc_info.code_hash = account.info.code_hash;
            let _ = self.set_account_info(address, acc_info);

            let is_newly_created = account.is_created();
            if is_newly_created {
                // TODO: can contract change other than creation??
                if let Some(code) = account.info.code {
                    let _ = self.set_code(account.info.code_hash, code);
                }
            }

            for (loc, slot) in account.storage {
                if !slot.is_changed() {
                    continue;
                }
                let _ = self.set_account_memory(address, loc, slot.present_value());
            }
        }
    }
}

/// Tests for all set and get methods
#[cfg(test)]
mod tests {
    use alloy_primitives::LogData;
    use revm::context::result::{Output, SuccessReason};
    use tempfile::TempDir;

    use super::*;
    use crate::db::types::BytesED;

    #[test]
    fn test_db() {
        let path = TempDir::new().unwrap().keep();

        let address = [1u8; 20].into();
        let code_hash = [2u8; 32].into();
        let bytecode = Bytecode::new_raw(vec![3u8; 32].into());
        let account_info = AccountInfo {
            balance: U256::from(100),
            nonce: 4,
            code_hash,
            code: Some(bytecode.clone()),
        };
        let mine_timestamp = 5;

        let mem_loc = U256::from(6);
        let value = U256::from(7);
        let block_number = 8;
        let block_hash = [9u8; 32].into();
        let block_timestamp = 10;
        let gas_used = 11;

        {
            let mut db = DB::new(&path).unwrap();

            db.set_account_info(address, account_info.clone()).unwrap();
            assert_eq!(
                db.get_account_info(address).unwrap().unwrap(),
                account_info.clone().into()
            );

            db.set_code(code_hash, bytecode.clone()).unwrap();
            assert_eq!(db.get_code(code_hash).unwrap().unwrap().bytecode, bytecode);

            db.set_account_memory(address, mem_loc, value).unwrap();
            assert_eq!(
                db.get_account_memory(address, mem_loc)
                    .unwrap()
                    .unwrap()
                    .uint,
                value
            );

            db.set_block_hash(block_number, block_hash).unwrap();
            assert_eq!(
                db.get_block_hash(block_number).unwrap().unwrap(),
                block_hash
            );

            db.set_block_timestamp(block_number, block_timestamp)
                .unwrap();
            assert_eq!(
                db.get_block_timestamp(block_number).unwrap().unwrap(),
                block_timestamp.try_into().unwrap()
            );

            db.set_gas_used(block_number, gas_used).unwrap();
            assert_eq!(
                db.get_gas_used(block_number).unwrap().unwrap(),
                gas_used.try_into().unwrap()
            );

            db.set_mine_timestamp(block_number, mine_timestamp).unwrap();
            assert_eq!(
                db.get_mine_timestamp(block_number).unwrap().unwrap(),
                mine_timestamp.try_into().unwrap()
            );

            db.commit_changes().unwrap();
        }

        let db = DB::new(&path).unwrap();

        assert_eq!(
            db.get_account_info(address).unwrap().unwrap(),
            account_info.into()
        );
        assert_eq!(db.get_code(code_hash).unwrap().unwrap().bytecode, bytecode);
        assert_eq!(
            db.get_account_memory(address, mem_loc)
                .unwrap()
                .unwrap()
                .uint,
            value
        );
        assert_eq!(
            db.get_block_hash(block_number).unwrap().unwrap(),
            block_hash
        );
        assert_eq!(
            db.get_block_timestamp(block_number).unwrap().unwrap(),
            block_timestamp.try_into().unwrap()
        );
        assert_eq!(
            db.get_gas_used(block_number).unwrap().unwrap(),
            gas_used.try_into().unwrap()
        );
        assert_eq!(
            db.get_mine_timestamp(block_number).unwrap().unwrap(),
            mine_timestamp.try_into().unwrap()
        );
    }

    #[test]
    fn test_tx_methods() {
        let path = TempDir::new().unwrap().keep();

        let data = vec![0u8; 32];
        let block_hash = [1u8; 32].into();
        let block_number = 2;
        let block_timestamp = 11;
        let contract_address = [3u8; 20].into();
        let from = [4u8; 20].into();
        let to = [5u8; 20].into();
        let tx_hash = [6u8; 32].into();
        let tx_idx = 7;
        let output = ExecutionResult::Success {
            reason: SuccessReason::Return,
            gas_used: 10,
            gas_refunded: 0,
            logs: Vec::new(),
            output: Output::Call(vec![11u8; 32].into()),
        };
        let cumulative_gas_used = 8;
        let nonce = 9;
        let start_log_index = 10;

        {
            let mut db = DB::new(&path).unwrap();

            db.set_tx_receipt(
                "type",
                "reason",
                Some(&vec![11u8; 32].into()),
                block_hash,
                block_number,
                block_timestamp,
                Some(contract_address),
                from,
                Some(to),
                &data.into(),
                tx_hash,
                tx_idx,
                &output,
                cumulative_gas_used,
                nonce,
                start_log_index,
                Some("inscription_id".to_string()),
                10000,
            )
            .unwrap();
            db.set_block_hash(block_number, block_hash).unwrap();

            db.commit_changes().unwrap();
        }

        let db = DB::new(&path).unwrap();

        assert_eq!(
            db.get_tx_hash_by_inscription_id("inscription_id".to_string())
                .unwrap()
                .unwrap()
                .bytes,
            tx_hash
        );
        assert_eq!(
            db.get_tx_hash_by_block_number_and_index(block_number, tx_idx)
                .unwrap()
                .unwrap()
                .bytes,
            tx_hash
        );
        assert_eq!(
            db.get_tx_hash_by_block_hash_and_index(block_hash, tx_idx)
                .unwrap()
                .unwrap()
                .bytes,
            tx_hash
        );
        assert_eq!(
            db.get_tx_receipt(tx_hash).unwrap().unwrap(),
            TxReceiptED::new(
                block_hash.into(),
                block_number.into(),
                block_timestamp.into(),
                Some(contract_address.into()),
                from.into(),
                Some(to.into()),
                tx_hash.into(),
                tx_idx.into(),
                &output,
                cumulative_gas_used.into(),
                nonce.into(),
                start_log_index.into(),
                "type".to_string(),
                "reason".to_string(),
                Some(BytesED::new(vec![11u8; 32].into())),
            )
            .unwrap()
        );
    }

    #[test]
    fn test_get_logs() {
        let path = TempDir::new().unwrap().keep();

        let block_number = 1;
        let block_hash = [1u8; 32].into();
        let block_timestamp = 11;
        let contract_address = [3u8; 20].into();
        let from = [4u8; 20].into();
        let to = [5u8; 20].into();
        let tx_hash = [6u8; 32].into();
        let tx_idx = 7;
        let output = ExecutionResult::Success {
            reason: SuccessReason::Return,
            gas_used: 10,
            gas_refunded: 0,
            logs: vec![
                Log::<LogData>::new(
                    contract_address,
                    vec![[7u8; 32].into(), [8u8; 32].into(), [9u8; 32].into()],
                    vec![10u8; 32].into(),
                )
                .unwrap(),
                Log::<LogData>::new(
                    contract_address,
                    vec![[10u8; 32].into(), [8u8; 32].into(), [11u8; 32].into()],
                    vec![11u8; 32].into(),
                )
                .unwrap(),
            ],
            output: Output::Call(vec![11u8; 32].into()),
        };
        let cumulative_gas_used = 8;
        let nonce = 9;
        let start_log_index = 10;
        let data = vec![0u8; 32];

        {
            let mut db = DB::new(&path).unwrap();

            db.set_tx_receipt(
                "type",
                "reason",
                Some(&vec![11u8; 32].into()),
                block_hash,
                block_number,
                block_timestamp,
                Some(contract_address),
                from,
                Some(to),
                &data.into(),
                tx_hash,
                tx_idx,
                &output,
                cumulative_gas_used,
                nonce,
                start_log_index,
                Some("inscription_id".to_string()),
                10000,
            )
            .unwrap();
            db.set_block_hash(block_number, block_hash).unwrap();

            db.commit_changes().unwrap();
        }

        let db = DB::new(&path).unwrap();

        let logs = db
            .get_logs(
                Some(block_number),
                Some(block_number),
                Some(contract_address),
                Some(vec![
                    SingleOrVec::Vec(vec![Some([7u8; 32].into()), Some([10u8; 32].into())]),
                    SingleOrVec::Single(Some([8u8; 32].into())),
                ]),
            )
            .unwrap();

        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].address.address, contract_address);
        assert_eq!(logs[0].topics.len(), 3);
        assert_eq!(logs[0].topics[0].bytes.0, [7u8; 32]);
        assert_eq!(logs[0].topics[1].bytes.0, [8u8; 32]);
        assert_eq!(logs[0].topics[2].bytes.0, [9u8; 32]);
        assert_eq!(logs[0].data.bytes.to_vec(), [10u8; 32].to_vec());
        assert_eq!(logs[1].address.address, contract_address);
        assert_eq!(logs[1].topics.len(), 3);
        assert_eq!(logs[1].topics[0].bytes.0, [10u8; 32]);
        assert_eq!(logs[1].topics[1].bytes.0, [8u8; 32]);
        assert_eq!(logs[1].topics[2].bytes.0, [11u8; 32]);
        assert_eq!(logs[1].data.bytes.to_vec(), [11u8; 32].to_vec());

        let logs = db
            .get_logs(
                Some(block_number),
                Some(block_number),
                Some(contract_address),
                Some(vec![
                    SingleOrVec::Single(None),
                    SingleOrVec::Single(None),
                    SingleOrVec::Single(Some([11u8; 32].into())),
                ]),
            )
            .unwrap();

        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].address.address, contract_address);
        assert_eq!(logs[0].topics.len(), 3);
        assert_eq!(logs[0].topics[0].bytes.0, [10u8; 32]);
        assert_eq!(logs[0].topics[1].bytes.0, [8u8; 32]);
        assert_eq!(logs[0].topics[2].bytes.0, [11u8; 32]);
        assert_eq!(logs[0].data.bytes.to_vec(), [11u8; 32].to_vec());
    }
}
