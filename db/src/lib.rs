// NOTE: Limbs are little-endian
// DB needs big-endian bytes

use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

use cached_database::BlockDatabase;
use heed::{Env, EnvOpenOptions, RwTxn};

use revm::primitives::alloy_primitives::U128;
use revm::primitives::alloy_primitives::U64;
use revm::primitives::db::Database as DatabaseTrait;
use revm::primitives::db::DatabaseCommit;
use revm::primitives::ruint::aliases::U256;
use revm::primitives::{Account, AccountInfo, Address, Bytecode, B256};

mod cached_database;
use cached_database::{BlockCachedDatabase, BlockHistoryCacheData};

mod test_utils;

mod types;
use types::U128ED;
use types::U64ED;
use types::{AccountInfoED, AddressED, BytecodeED, B256ED, U256ED, U512ED};

pub struct DB {
    env: Option<Env>,
    // Account address to memory location
    db_account_memory: Option<BlockCachedDatabase<U512ED, U256ED, BlockHistoryCacheData<U256ED>>>,

    // Code hash to bytecode
    db_code: Option<BlockCachedDatabase<B256ED, BytecodeED, BlockHistoryCacheData<BytecodeED>>>,

    // Account address to account info
    db_account:
        Option<BlockCachedDatabase<AddressED, AccountInfoED, BlockHistoryCacheData<AccountInfoED>>>,

    // Block hash to block number
    db_block_hash_to_number:
        Option<BlockCachedDatabase<B256ED, U64ED, BlockHistoryCacheData<U64ED>>>,

    // Block number to block hash
    db_block_number_to_hash: Option<BlockDatabase<B256ED>>,

    // Block number to block timestamp
    db_block_number_to_timestamp: Option<BlockDatabase<U64ED>>,

    // Block number to gas used
    db_block_number_to_gas_used: Option<BlockDatabase<U64ED>>,

    // Block number to mine timestamp
    db_block_number_to_mine_tm: Option<BlockDatabase<U128ED>>,

    // Cache for latest block number and block hash
    latest_block_number: Option<(u64, B256)>,
}

impl Default for DB {
    fn default() -> Self {
        Self {
            env: None,
            db_account_memory: None,
            db_code: None,
            db_account: None,
            db_block_number_to_hash: None,
            db_block_hash_to_number: None,
            db_block_number_to_timestamp: None,
            db_block_number_to_gas_used: None,
            db_block_number_to_mine_tm: None,
            latest_block_number: None,
        }
    }
}

fn create_env() -> Result<Env, Box<dyn Error>> {
    let path = Path::new("target").join("heed.mdb");
    fs::create_dir_all(&path)?;

    let env = unsafe {
        EnvOpenOptions::new()
            .map_size(20 * 1024 * 1024 * 1024) // 20GB // TODO: set this reasonably!!
            .max_dbs(3000)
            .open(path)?
    };

    Ok(env)
}

impl DB {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let env = create_env()?;

        let mut wtxn = env.write_txn()?;
        let db_account_memory = Some(BlockCachedDatabase::new(
            env.clone(),
            "account_memory_map",
            &mut wtxn,
        ));
        let db_code = Some(BlockCachedDatabase::new(env.clone(), "code_map", &mut wtxn));
        let db_account = Some(BlockCachedDatabase::new(
            env.clone(),
            "account_map",
            &mut wtxn,
        ));
        let db_block_hash_to_number = Some(BlockCachedDatabase::new(
            env.clone(),
            "block_hash_to_number",
            &mut wtxn,
        ));

        let db_block_number_to_hash =
            Some(BlockDatabase::new(env.clone(), "block_hash", &mut wtxn));
        let db_block_number_to_timestamp =
            Some(BlockDatabase::new(env.clone(), "block_ts", &mut wtxn));
        let db_block_number_to_gas_used =
            Some(BlockDatabase::new(env.clone(), "gas_used", &mut wtxn));
        let db_block_number_to_mine_tm =
            Some(BlockDatabase::new(env.clone(), "mine_tm", &mut wtxn));

        wtxn.commit()?;

        Ok(Self {
            env: Some(env),
            db_account_memory,
            db_code,
            db_account,
            db_block_number_to_hash,
            db_block_hash_to_number,
            db_block_number_to_timestamp,
            db_block_number_to_gas_used,
            db_block_number_to_mine_tm,
            latest_block_number: None,
        })
    }

    pub fn get_write_txn(&self) -> Result<RwTxn, Box<dyn Error>> {
        Ok(self.env.as_ref().unwrap().write_txn()?)
    }

    pub fn get_latest_block_height(&self) -> Result<u64, Box<dyn Error>> {
        if self.latest_block_number.is_some() {
            return Ok(self.latest_block_number.unwrap().0);
        }
        self.db_block_number_to_hash
            .as_ref()
            .unwrap()
            .last_key()
            .ok_or_else(|| "Latest block number not found".into())
    }

    pub fn get_account_memory(
        &mut self,
        account: Address,
        mem_loc: U256,
    ) -> Result<Option<U256>, Box<dyn Error>> {
        let ret = self
            .db_account_memory
            .as_ref()
            .unwrap()
            .latest(&U512ED::from_addr_u256(account, mem_loc));

        Ok(ret.map(|x| x.0))
    }

    pub fn set_account_memory(
        &mut self,
        account: Address,
        mem_loc: U256,
        value: U256,
    ) -> Result<(), Box<dyn Error>> {
        let block_number = self.get_latest_block_height()?;
        self.db_account_memory.as_mut().unwrap().set(
            block_number,
            U512ED::from_addr_u256(account, mem_loc),
            U256ED::from_u256(value),
        )?;

        Ok(())
    }

    pub fn get_code(&mut self, code_hash: B256) -> Result<Option<Bytecode>, Box<dyn Error>> {
        let ret = self
            .db_code
            .as_ref()
            .unwrap()
            .latest(&B256ED::from_b256(code_hash));

        Ok(ret.map(|x| x.0))
    }

    pub fn set_code(&mut self, code_hash: B256, bytecode: Bytecode) -> Result<(), Box<dyn Error>> {
        let block_number = self.get_latest_block_height()?;
        self.db_code.as_mut().unwrap().set(
            block_number,
            B256ED::from_b256(code_hash),
            BytecodeED::from_bytecode(bytecode),
        )?;
        Ok(())
    }

    pub fn get_account_info(
        &mut self,
        account: Address,
    ) -> Result<Option<AccountInfo>, Box<dyn Error>> {
        let ret = self
            .db_account
            .as_ref()
            .unwrap()
            .latest(&AddressED::from_addr(account));

        Ok(ret.map(|x| x.0))
    }

    pub fn set_account_info(
        &mut self,
        account: Address,
        value: AccountInfo,
    ) -> Result<(), Box<dyn Error>> {
        let block_number = self.get_latest_block_height()?;
        self.db_account.as_mut().unwrap().set(
            block_number,
            AddressED::from_addr(account),
            AccountInfoED::from_account_info(value),
        )?;

        Ok(())
    }

    pub fn get_block_hash(&mut self, block_number: u64) -> Result<Option<B256>, Box<dyn Error>> {
        let ret = self
            .db_block_number_to_hash
            .as_mut()
            .unwrap()
            .get(block_number);

        Ok(ret.map(|x| x.0))
    }

    pub fn set_block_hash(
        &mut self,
        block_number: u64,
        block_hash: B256,
    ) -> Result<(), Box<dyn Error>> {
        if block_number > self.latest_block_number.unwrap_or((0, B256::ZERO)).0 {
            self.latest_block_number = Some((block_number, block_hash));
        }

        self.db_block_number_to_hash
            .as_mut()
            .unwrap()
            .set(block_number, B256ED::from_b256(block_hash));
        self.db_block_hash_to_number.as_mut().unwrap().set(
            block_number,
            B256ED::from_b256(block_hash),
            U64ED::from_u64(block_number),
        )?;

        Ok(())
    }

    pub fn get_block_timestamp(&mut self, number: u64) -> Result<Option<U64>, Box<dyn Error>> {
        let ret = self
            .db_block_number_to_timestamp
            .as_mut()
            .unwrap()
            .get(number);

        Ok(ret.map(|x| x.0))
    }

    pub fn set_block_timestamp(
        &mut self,
        block_number: u64,
        block_timestamp: u64,
    ) -> Result<(), Box<dyn Error>> {
        self.db_block_number_to_timestamp
            .as_mut()
            .unwrap()
            .set(block_number, U64ED::from_u64(block_timestamp));

        Ok(())
    }

    pub fn get_gas_used(&mut self, block_number: u64) -> Result<Option<U64>, Box<dyn Error>> {
        let ret = self
            .db_block_number_to_gas_used
            .as_mut()
            .unwrap()
            .get(block_number);

        Ok(ret.map(|x| x.0))
    }

    pub fn set_gas_used(&mut self, block_number: u64, gas_used: u64) -> Result<(), Box<dyn Error>> {
        self.db_block_number_to_gas_used
            .as_mut()
            .unwrap()
            .set(block_number, U64ED::from_u64(gas_used));

        Ok(())
    }

    pub fn get_mine_timestamp(
        &mut self,
        block_number: u64,
    ) -> Result<Option<U128>, Box<dyn Error>> {
        let ret = self
            .db_block_number_to_mine_tm
            .as_mut()
            .unwrap()
            .get(block_number);

        Ok(ret.map(|x| x.0))
    }

    pub fn set_mine_timestamp(
        &mut self,
        block_number: u64,
        mine_timestamp: u128,
    ) -> Result<(), Box<dyn Error>> {
        self.db_block_number_to_mine_tm
            .as_mut()
            .unwrap()
            .set(block_number, U128ED::from_u128(mine_timestamp));

        Ok(())
    }

    pub fn commit_changes(&mut self) -> Result<(), Box<dyn Error>> {
        let env = self.env.clone().unwrap();
        let mut wtxn = env.write_txn()?;

        self.db_block_number_to_hash
            .as_mut()
            .unwrap()
            .commit(&mut wtxn);
        self.db_block_number_to_timestamp
            .as_mut()
            .unwrap()
            .commit(&mut wtxn);
        self.db_block_number_to_gas_used
            .as_mut()
            .unwrap()
            .commit(&mut wtxn);
        self.db_block_number_to_mine_tm
            .as_mut()
            .unwrap()
            .commit(&mut wtxn);

        self.db_account_memory.as_ref().unwrap().commit(&mut wtxn)?;
        self.db_code.as_ref().unwrap().commit(&mut wtxn)?;
        self.db_account.as_ref().unwrap().commit(&mut wtxn)?;
        self.db_block_hash_to_number
            .as_ref()
            .unwrap()
            .commit(&mut wtxn)?;
        wtxn.commit()?;
        self.env.clone().unwrap().force_sync()?;

        self.clear_caches();
        Ok(())
    }

    pub fn clear_caches(&mut self) {
        self.db_account_memory.as_mut().unwrap().clear_cache();
        self.db_code.as_mut().unwrap().clear_cache();
        self.db_account.as_mut().unwrap().clear_cache();
        self.db_block_number_to_hash.as_mut().unwrap().clear_cache();
        self.db_block_hash_to_number.as_mut().unwrap().clear_cache();
        self.db_block_number_to_timestamp
            .as_mut()
            .unwrap()
            .clear_cache();
        self.db_block_number_to_gas_used
            .as_mut()
            .unwrap()
            .clear_cache();
        self.db_block_number_to_mine_tm
            .as_mut()
            .unwrap()
            .clear_cache();
    }

    pub fn reorg(&mut self, latest_valid_block_number: u64) -> Result<(), Box<dyn Error>> {
        let env = self.env.clone().unwrap();
        let mut wtxn = env.write_txn()?;

        self.db_account_memory
            .as_mut()
            .unwrap()
            .reorg(&mut wtxn, latest_valid_block_number)?;
        self.db_code
            .as_mut()
            .unwrap()
            .reorg(&mut wtxn, latest_valid_block_number)?;
        self.db_account
            .as_mut()
            .unwrap()
            .reorg(&mut wtxn, latest_valid_block_number)?;
        self.db_block_hash_to_number
            .as_mut()
            .unwrap()
            .reorg(&mut wtxn, latest_valid_block_number)?;

        self.db_block_number_to_hash
            .as_mut()
            .unwrap()
            .reorg(&mut wtxn, latest_valid_block_number);
        self.db_block_number_to_timestamp
            .as_mut()
            .unwrap()
            .reorg(&mut wtxn, latest_valid_block_number);
        self.db_block_number_to_gas_used
            .as_mut()
            .unwrap()
            .reorg(&mut wtxn, latest_valid_block_number);
        self.db_block_number_to_mine_tm
            .as_mut()
            .unwrap()
            .reorg(&mut wtxn, latest_valid_block_number);

        wtxn.commit()?;
        self.env.clone().unwrap().force_sync()?;
        self.clear_caches();

        Ok(())
    }
}

impl DatabaseTrait for DB {
    type Error = Box<dyn Error>;

    /// Get basic account information.
    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        // println!("basic {}", address);
        let res = self.get_account_info(address)?;
        // println!("basic res {:?}", res);

        if res.is_some() {
            let mut res = res.unwrap();
            res.code = Some(self.code_by_hash(res.code_hash).unwrap());
            Ok(Some(res))
        } else {
            Ok(res)
        }
    }

    /// Get account code by its hash.
    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        // println!("code_by_hash {}", code_hash);
        self.get_code(code_hash)
            .map(|x| x.unwrap_or(Bytecode::default()))
    }

    /// Get storage value of address at index.
    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        // println!("storage {} {}", address, index);
        self.get_account_memory(address, index)
            .map(|x| x.unwrap_or(U256::ZERO))
    }

    /// Get block hash by block number.
    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        // println!("block_hash {}", number);
        self.get_block_hash(number).map(|x| x.unwrap_or(B256::ZERO))
    }
}

impl DatabaseCommit for DB {
    fn commit(&mut self, changes: HashMap<Address, Account>) {
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
                let _ = self.set_code(account.info.code_hash, account.info.code.unwrap());
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
