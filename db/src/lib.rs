// NOTE: Limbs are little-endian
// DB needs big-endian bytes

use std::error::Error;
use std::fs;
use std::path::Path;

use heed::{Database, Env, EnvOpenOptions, RwTxn};

use revm::primitives::db::Database as DatabaseTrait;
use revm::primitives::db::DatabaseCommit;
use revm::primitives::ruint::aliases::U256;
use revm::primitives::ruint::Uint;
use revm::primitives::{Account, AccountInfo, Address, Bytecode, B256};

use hashbrown::HashMap as Map;

mod cache;
use cache::BlockCachedDatabase;

mod test_utils;

mod types;
use types::{AccountInfoED, AddressED, BytecodeED, CacheVal, B256ED, U256ED, U512ED};

pub struct DB {
    env: Option<Env>,
    // Account address to memory location
    db_account_memory: Option<Database<U512ED, U256ED>>,
    // In memory cache for account memory
    account_memory_cache: Map<Address, Map<U256, CacheVal<U256>>>,

    // Code hash to bytecode
    db_code: Option<Database<B256ED, BytecodeED>>,
    // In memory cache for code
    code_cache: Map<B256, Bytecode>,

    // Account address to account info
    db_account: Option<Database<AddressED, AccountInfoED>>,
    // In memory cache for account info
    account_cache: Map<Address, CacheVal<AccountInfo>>,

    // Block number to block hash
    db_block_number_to_hash: Option<Database<U256ED, B256ED>>,
    // In memory cache for block hash
    block_number_to_hash_cache: Map<U256, CacheVal<B256>>,
    // Block hash to block number
    db_block_hash_to_number: Option<Database<B256ED, U256ED>>,
    // In memory cache for block_number_hash_map_reverse
    block_hash_to_number_cache: Map<B256, CacheVal<U256>>,

    // Block number to block timestamp
    db_block_number_to_timestamp: Option<Database<U256ED, U256ED>>,
    // In memory cache for block timestamps
    block_number_to_timestamp_cache: Map<U256, CacheVal<U256>>,

    // Block number to gas used
    db_block_number_to_gas_used: Option<Database<U256ED, U256ED>>,

    // Block number to mine timestamp
    db_block_number_to_mine_tm: Option<Database<U256ED, U256ED>>,

    // Cache for latest block number and block hash
    latest_block_hash: Option<(U256, B256)>,
}

impl DB {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let path = Path::new("target").join("heed.mdb");
        fs::create_dir_all(&path)?;

        let env = EnvOpenOptions::new()
            .map_size(20 * 1024 * 1024 * 1024) // 20GB // TODO: set this reasonably!!
            .max_dbs(3000)
            .open(path)?;

        let mut wtxn = env.write_txn()?;
        let db_account_memory_map: Database<U512ED, U256ED> = {
            let old_db = env.open_database::<U512ED, U256ED>(Some("account_memory_map"))?;
            if old_db.is_some() {
                old_db.unwrap()
            } else {
                env.create_database_with_txn(Some("account_memory_map"), &mut wtxn)?
            }
        };
        let db_code_map: Database<B256ED, BytecodeED> = {
            let old_db = env.open_database::<B256ED, BytecodeED>(Some("code_map"))?;
            if old_db.is_some() {
                old_db.unwrap()
            } else {
                env.create_database_with_txn(Some("code_map"), &mut wtxn)?
            }
        };
        let db_account_map: Database<AddressED, AccountInfoED> = {
            let old_db = env.open_database::<AddressED, AccountInfoED>(Some("account_map"))?;
            if old_db.is_some() {
                old_db.unwrap()
            } else {
                env.create_database_with_txn(Some("account_map"), &mut wtxn)?
            }
        };
        let db_block_hash_map: Database<U256ED, B256ED> = {
            let old_db = env.open_database::<U256ED, B256ED>(Some("block_hash"))?;
            if old_db.is_some() {
                old_db.unwrap()
            } else {
                env.create_database_with_txn(Some("block_hash"), &mut wtxn)?
            }
        };
        let db_block_hash_to_number: Database<B256ED, U256ED> = {
            let old_db = env.open_database::<B256ED, U256ED>(Some("block_hash_to_number"))?;
            if old_db.is_some() {
                old_db.unwrap()
            } else {
                env.create_database_with_txn(Some("block_hash_to_number"), &mut wtxn)?
            }
        };
        let db_block_timestamp_map: Database<U256ED, U256ED> = {
            let old_db = env.open_database::<U256ED, U256ED>(Some("block_ts"))?;
            if old_db.is_some() {
                old_db.unwrap()
            } else {
                env.create_database_with_txn(Some("block_ts"), &mut wtxn)?
            }
        };
        let db_block_gas_used_map: Database<U256ED, U256ED> = {
            let old_db = env.open_database::<U256ED, U256ED>(Some("gas_used"))?;
            if old_db.is_some() {
                old_db.unwrap()
            } else {
                env.create_database_with_txn(Some("gas_used"), &mut wtxn)?
            }
        };
        let db_block_mine_tm_map: Database<U256ED, U256ED> = {
            let old_db = env.open_database::<U256ED, U256ED>(Some("mine_tm"))?;
            if old_db.is_some() {
                old_db.unwrap()
            } else {
                env.create_database_with_txn(Some("mine_tm"), &mut wtxn)?
            }
        };
        wtxn.commit()?;

        Ok(Self {
            env: Some(env),
            db_account_memory: Some(db_account_memory_map),
            db_code: Some(db_code_map),
            db_account: Some(db_account_map),
            db_block_number_to_hash: Some(db_block_hash_map),
            db_block_hash_to_number: Some(db_block_hash_to_number),
            db_block_number_to_timestamp: Some(db_block_timestamp_map),
            db_block_number_to_gas_used: Some(db_block_gas_used_map),
            db_block_number_to_mine_tm: Some(db_block_mine_tm_map),
            code_cache: Map::new(),
            account_memory_cache: Map::new(),
            account_cache: Map::new(),
            block_number_to_hash_cache: Map::new(),
            block_hash_to_number_cache: Map::new(),
            block_number_to_timestamp_cache: Map::new(),
            latest_block_hash: None,
        })
    }

    pub fn get_write_txn(&self) -> Result<RwTxn, Box<dyn Error>> {
        Ok(self.env.as_ref().unwrap().write_txn()?)
    }

    pub fn read_from_account_memory_map(
        &mut self,
        account: Address,
        mem_loc: U256,
    ) -> Result<Option<U256>, Box<dyn Error>> {
        if !self.account_memory_cache.contains_key(&account) {
            self.account_memory_cache.insert(account, Map::new());
        }
        let acc = self.account_memory_cache.get_mut(&account).unwrap();
        if acc.contains_key(&mem_loc) {
            return Ok(acc.get(&mem_loc).unwrap().get_current());
        }

        let rtxn = self.env.as_ref().unwrap().read_txn()?;
        let ret: Option<U256ED> = self
            .db_account_memory
            .unwrap()
            .get(&rtxn, &U512ED::from_addr_u256(account, mem_loc))?;

        if ret.is_some() {
            acc.insert(mem_loc, CacheVal::new_not_changed(&ret.as_ref().unwrap().0));
        }

        Ok(ret.map(|x| x.0))
    }

    pub fn set_account_memory_map(
        &mut self,
        account: Address,
        mem_loc: U256,
        value: U256,
    ) -> Result<(), Box<dyn Error>> {
        if !self.account_memory_cache.contains_key(&account) {
            self.account_memory_cache.insert(account, Map::new());
        }
        let acc = self.account_memory_cache.get_mut(&account).unwrap();
        if acc.contains_key(&mem_loc) {
            let cached_val = acc.get_mut(&mem_loc).unwrap();
            cached_val.set_current(&Some(value));
            return Ok(());
        }

        let rtxn = self.env.as_ref().unwrap().read_txn()?;
        let ret: Option<U256ED> = self
            .db_account_memory
            .unwrap()
            .get(&rtxn, &U512ED::from_addr_u256(account, mem_loc))?;

        if ret.is_some() {
            acc.insert(
                mem_loc,
                CacheVal::new_changed(&ret.unwrap().0, &Some(value)),
            );
        } else {
            acc.insert(mem_loc, CacheVal::new_created(&value));
        }

        Ok(())
    }

    pub fn remove_from_account_memory_map(
        &mut self,
        account: Address,
    ) -> Result<(), Box<dyn Error>> {
        if !self.account_memory_cache.contains_key(&account) {
            self.account_memory_cache.insert(account, Map::new());
        }
        let acc = self.account_memory_cache.get_mut(&account).unwrap();

        for elem in acc.iter_mut() {
            elem.1.set_current(&None);
        }

        let min_loc = U512ED::from_addr_u256(account, U256::ZERO);
        let max_loc = U512ED::from_addr_u256(account, U256::MAX);
        let range = min_loc..=max_loc;

        let rtxn = self.env.as_ref().unwrap().read_txn()?;
        let olds_iter = self
            .db_account_memory
            .unwrap()
            .range(&rtxn, &range)
            .unwrap();
        for elem in olds_iter {
            let key = elem.as_ref().unwrap().0.clone();
            let value = elem.unwrap().1;
            let mem_loc = get_lower_u256(key);
            acc.insert(mem_loc, CacheVal::new_changed(&(value.0), &None));
        }

        Ok(())
    }

    pub fn read_from_code_map(
        &mut self,
        code_hash: B256,
    ) -> Result<Option<Bytecode>, Box<dyn Error>> {
        if self.code_cache.contains_key(&code_hash) {
            return Ok(Some(self.code_cache.get(&code_hash).unwrap().clone()));
        }

        let rtxn = self.env.as_ref().unwrap().read_txn()?;

        let ret: Option<BytecodeED> = self
            .db_code
            .unwrap()
            .get(&rtxn, &B256ED::from_b256(code_hash))?;

        if ret.is_some() {
            let v = ret.unwrap().0;
            self.code_cache.insert(code_hash, v.clone());
            return Ok(Some(v));
        }

        Ok(ret.map(|x| x.0))
    }

    pub fn set_code_map_with_txn(
        &self,
        code_hash: B256,
        bytecode: Bytecode,
        parent_wtxn: &mut RwTxn,
    ) -> Result<(), Box<dyn Error>> {
        self.db_code.unwrap().put(
            parent_wtxn,
            &B256ED::from_b256(code_hash),
            &BytecodeED::from_bytecode(bytecode),
        )?;
        Ok(())
    }

    pub fn read_from_account_map(
        &mut self,
        account: Address,
    ) -> Result<Option<AccountInfo>, Box<dyn Error>> {
        if self.account_cache.contains_key(&account) {
            return Ok(self.account_cache.get(&account).unwrap().get_current());
        }

        let rtxn = self.env.as_ref().unwrap().read_txn()?;
        let ret: Option<AccountInfoED> = self
            .db_account
            .unwrap()
            .get(&rtxn, &AddressED::from_addr(account))?;

        if ret.is_some() {
            self.account_cache
                .insert(account, CacheVal::new_not_changed(&ret.as_ref().unwrap().0));
        }

        Ok(ret.map(|x| x.0))
    }

    pub fn set_account_map(
        &mut self,
        account: Address,
        value: AccountInfo,
    ) -> Result<(), Box<dyn Error>> {
        if self.account_cache.contains_key(&account) {
            let cached_val = self.account_cache.get_mut(&account).unwrap();
            cached_val.set_current(&Some(value));
            return Ok(());
        }

        let rtxn = self.env.as_ref().unwrap().read_txn()?;
        let ret: Option<AccountInfoED> = self
            .db_account
            .unwrap()
            .get(&rtxn, &AddressED::from_addr(account))?;

        if ret.is_some() {
            self.account_cache.insert(
                account,
                CacheVal::new_changed(&ret.unwrap().0, &Some(value)),
            );
        } else {
            self.account_cache
                .insert(account, CacheVal::new_created(&value));
        }

        Ok(())
    }

    pub fn remove_from_account_map(&mut self, account: Address) -> Result<(), Box<dyn Error>> {
        if self.account_cache.contains_key(&account) {
            let cached_val = self.account_cache.get_mut(&account).unwrap();
            cached_val.set_current(&None);
            return Ok(());
        }

        let rtxn = self.env.as_ref().unwrap().read_txn()?;
        let ret: Option<AccountInfoED> = self
            .db_account
            .unwrap()
            .get(&rtxn, &AddressED::from_addr(account))?;

        if ret.is_some() {
            self.account_cache
                .insert(account, CacheVal::new_changed(&ret.unwrap().0, &None));
        } else {
            panic!("REMOVED NON EXISTING ACCOUNT!!") // TODO: check if this really cannot happen!!
                                                     // account_cache.insert(account, CacheVal::new_created(&None));
        }

        Ok(())
    }

    pub fn read_from_block_hashes(&mut self, number: U256) -> Result<Option<B256>, Box<dyn Error>> {
        // Check if the caches for the block hash and block number match, otherwise read from DB
        if self.block_number_to_hash_cache.contains_key(&number) {
            let cached_hash = self.block_number_to_hash_cache.get(&number).unwrap();
            let cached_number = self
                .block_hash_to_number_cache
                .get(&cached_hash.get_current().unwrap())
                .unwrap();
            if cached_number.get_current().unwrap() == number {
                return Ok(cached_hash.get_current());
            }
        }

        let rtxn = self.env.as_ref().unwrap().read_txn()?;
        let ret: Option<B256ED> = self
            .db_block_number_to_hash
            .unwrap()
            .get(&rtxn, &U256ED::from_u256(number))?;

        if ret.is_some() {
            self.block_number_to_hash_cache
                .insert(number, CacheVal::new_not_changed(&ret.as_ref().unwrap().0));
            self.block_hash_to_number_cache
                .insert(ret.as_ref().unwrap().0, CacheVal::new_not_changed(&number));
        }

        Ok(ret.map(|x| x.0))
    }

    pub fn set_block_hash(&mut self, number: U256, value: B256) -> Result<(), Box<dyn Error>> {
        if self.block_number_to_hash_cache.contains_key(&number)
            && self.block_hash_to_number_cache.contains_key(&value)
        {
            let cached_hash = self.block_number_to_hash_cache.get_mut(&number).unwrap();
            let old_hash = cached_hash.get_current();
            if old_hash.is_some() {
                let cached_number = self
                    .block_hash_to_number_cache
                    .get_mut(&old_hash.unwrap())
                    .unwrap();
                cached_number.set_current(&None);
            }
            cached_hash.set_current(&Some(value));
            let cached_number = self.block_hash_to_number_cache.get_mut(&value).unwrap();
            cached_number.set_current(&Some(number));
            return Ok(());
        }

        // fill block_hash_cache
        let rtxn = self.env.as_ref().unwrap().read_txn()?;
        let ret: Option<B256ED> = self
            .db_block_number_to_hash
            .unwrap()
            .get(&rtxn, &U256ED::from_u256(number))?;

        if ret.is_some() {
            self.block_number_to_hash_cache
                .insert(number, CacheVal::new_changed(&ret.unwrap().0, &Some(value)));
        } else {
            self.block_number_to_hash_cache
                .insert(number, CacheVal::new_created(&value));
        }

        // fill block_hash_to_number_cache
        let ret: Option<U256ED> = self
            .db_block_hash_to_number
            .unwrap()
            .get(&rtxn, &B256ED::from_b256(value))?;

        if ret.is_some() {
            self.block_hash_to_number_cache
                .insert(value, CacheVal::new_changed(&ret.unwrap().0, &Some(number)));
        } else {
            self.block_hash_to_number_cache
                .insert(value, CacheVal::new_created(&number));
        }

        if self.latest_block_hash.is_none() {
            self.latest_block_hash = Some((number, value));
        } else {
            let old_number = self.latest_block_hash.unwrap().0;
            if number > old_number {
                // println!("Updating latest block hash with block number {}", number);
                self.latest_block_hash = Some((number, value));
            }
        }

        Ok(())
    }

    pub fn get_latest_block_hash(&mut self) -> Result<Option<(U256, B256)>, Box<dyn Error>> {
        if self.latest_block_hash.is_some() {
            return Ok(self.latest_block_hash);
        }

        let rtxn = self.env.as_ref().unwrap().read_txn()?;
        let ret: Option<(U256ED, B256ED)> = self.db_block_number_to_hash.unwrap().last(&rtxn)?;

        if ret.is_some() {
            self.latest_block_hash = ret.map(|x| (x.0 .0, x.1 .0))
        }

        Ok(self.latest_block_hash)
    }

    pub fn read_from_block_timestamps(
        &mut self,
        number: U256,
    ) -> Result<Option<U256>, Box<dyn Error>> {
        if self.block_number_to_timestamp_cache.contains_key(&number) {
            return Ok(self
                .block_number_to_timestamp_cache
                .get(&number)
                .unwrap()
                .get_current());
        }

        let rtxn = self.env.as_ref().unwrap().read_txn()?;
        let ret: Option<U256ED> = self
            .db_block_number_to_timestamp
            .unwrap()
            .get(&rtxn, &U256ED::from_u256(number))?;

        if ret.is_some() {
            self.block_number_to_timestamp_cache
                .insert(number, CacheVal::new_not_changed(&ret.as_ref().unwrap().0));
        }

        Ok(ret.map(|x| x.0))
    }

    pub fn set_block_timestamp(&mut self, number: U256, value: U256) -> Result<(), Box<dyn Error>> {
        if self.block_number_to_timestamp_cache.contains_key(&number) {
            let cached_val = self
                .block_number_to_timestamp_cache
                .get_mut(&number)
                .unwrap();
            cached_val.set_current(&Some(value));
            return Ok(());
        }

        let rtxn = self.env.as_ref().unwrap().read_txn()?;
        let ret: Option<U256ED> = self
            .db_block_number_to_timestamp
            .unwrap()
            .get(&rtxn, &U256ED::from_u256(number))?;

        if ret.is_some() {
            self.block_number_to_timestamp_cache
                .insert(number, CacheVal::new_changed(&ret.unwrap().0, &Some(value)));
        } else {
            self.block_number_to_timestamp_cache
                .insert(number, CacheVal::new_created(&value));
        }

        Ok(())
    }

    pub fn read_from_block_gas_used(&self, number: U256) -> Result<Option<U256>, Box<dyn Error>> {
        let rtxn = self.env.as_ref().unwrap().read_txn()?;

        let ret: Option<U256ED> = self
            .db_block_number_to_gas_used
            .unwrap()
            .get(&rtxn, &U256ED::from_u256(number))?;
        Ok(ret.map(|x| x.0))
    }

    pub fn set_block_gas_used_with_txn(
        &self,
        number: U256,
        value: U256,
        parent_wtxn: &mut RwTxn,
    ) -> Result<(), Box<dyn Error>> {
        self.db_block_number_to_gas_used.unwrap().put(
            parent_wtxn,
            &U256ED::from_u256(number),
            &U256ED::from_u256(value),
        )?;
        Ok(())
    }

    pub fn read_from_block_mine_tm(&self, number: U256) -> Result<Option<U256>, Box<dyn Error>> {
        let rtxn = self.env.as_ref().unwrap().read_txn()?;

        let ret: Option<U256ED> = self
            .db_block_number_to_mine_tm
            .unwrap()
            .get(&rtxn, &U256ED::from_u256(number))?;
        Ok(ret.map(|x| x.0))
    }

    pub fn set_block_mine_tm_with_txn(
        &self,
        number: U256,
        value: U256,
        parent_wtxn: &mut RwTxn,
    ) -> Result<(), Box<dyn Error>> {
        self.db_block_number_to_mine_tm.unwrap().put(
            parent_wtxn,
            &U256ED::from_u256(number),
            &U256ED::from_u256(value),
        )?;
        Ok(())
    }

    pub fn commit_changes_to_db(&mut self) {
        // TODO: persist caches to DB for reorg to work
        let mut wtxn = self.get_write_txn().unwrap();
        for acc in self.account_memory_cache.iter() {
            let account = acc.0;
            for mem_slot in acc.1.iter() {
                if !mem_slot.1.is_changed() {
                    continue;
                }

                let mem_loc = mem_slot.0;
                let value = mem_slot.1.get_current();

                if value.is_some() {
                    self.db_account_memory
                        .unwrap()
                        .put(
                            &mut wtxn,
                            &U512ED::from_addr_u256(*account, *mem_loc),
                            &U256ED::from_u256(value.unwrap()),
                        )
                        .unwrap();
                } else {
                    self.db_account_memory
                        .unwrap()
                        .delete(&mut wtxn, &U512ED::from_addr_u256(*account, *mem_loc))
                        .unwrap();
                }
            }
        }

        for mem_slot in self.account_cache.iter() {
            if !mem_slot.1.is_changed() {
                continue;
            }

            let account = mem_slot.0;
            let value = mem_slot.1.get_current();

            if value.is_some() {
                self.db_account
                    .unwrap()
                    .put(
                        &mut wtxn,
                        &AddressED::from_addr(*account),
                        &AccountInfoED::from_account_info(value.unwrap()),
                    )
                    .unwrap();
            } else {
                self.db_account
                    .unwrap()
                    .delete(&mut wtxn, &AddressED::from_addr(*account))
                    .unwrap();
            }
        }

        for mem_slot in self.block_number_to_hash_cache.iter() {
            if !mem_slot.1.is_changed() {
                continue;
            }

            let number = mem_slot.0;
            let value = mem_slot.1.get_current();

            if value.is_some() {
                self.db_block_number_to_hash
                    .unwrap()
                    .put(
                        &mut wtxn,
                        &U256ED::from_u256(*number),
                        &B256ED::from_b256(value.unwrap()),
                    )
                    .unwrap();
            } else {
                self.db_block_number_to_hash
                    .unwrap()
                    .delete(&mut wtxn, &U256ED::from_u256(*number))
                    .unwrap();
            }
        }

        for mem_slot in self.block_hash_to_number_cache.iter() {
            if !mem_slot.1.is_changed() {
                continue;
            }

            let hash = mem_slot.0;
            let value = mem_slot.1.get_current();

            if value.is_some() {
                self.db_block_hash_to_number
                    .unwrap()
                    .put(
                        &mut wtxn,
                        &B256ED::from_b256(*hash),
                        &U256ED::from_u256(value.unwrap()),
                    )
                    .unwrap();
            } else {
                self.db_block_hash_to_number
                    .unwrap()
                    .delete(&mut wtxn, &B256ED::from_b256(*hash))
                    .unwrap();
            }
        }

        for mem_slot in self.block_number_to_timestamp_cache.iter() {
            if !mem_slot.1.is_changed() {
                continue;
            }

            let number = mem_slot.0;
            let value = mem_slot.1.get_current();

            if value.is_some() {
                self.db_block_number_to_timestamp
                    .unwrap()
                    .put(
                        &mut wtxn,
                        &U256ED::from_u256(*number),
                        &U256ED::from_u256(value.unwrap()),
                    )
                    .unwrap();
            } else {
                self.db_block_number_to_timestamp
                    .unwrap()
                    .delete(&mut wtxn, &U256ED::from_u256(*number))
                    .unwrap();
            }
        }
        wtxn.commit().unwrap();

        self.clear_caches();
    }

    pub fn clear_caches(&mut self) {
        self.account_memory_cache.clear();
        self.account_cache.clear();
        self.block_number_to_hash_cache.clear();
        self.block_hash_to_number_cache.clear();
        self.block_number_to_timestamp_cache.clear();
        self.latest_block_hash = None;
    }

    pub fn reorg(&mut self, _last_correct_block: U256) {
        // TODO: implement reorg
        // check if last_correct_block is in the DB for an early return
        // check distance between last_correct_block and latest_block_hash and if it's more than X blocks, then return

        // - iterate over the block_hash_cache and remove all blocks after last_correct_block
        // - iterate over the account_memory_cache and remove all accounts after last_correct_block
        // - iterate over the account_cache and remove all accounts after last_correct_block
        // - iterate over the block_timestamp_cache and remove all blocks after last_correct_block
        // - iterate over the block_gas_used_cache and remove all blocks after last_correct_block
        // - iterate over the block_mine_tm_cache and remove all blocks after last_correct_block

        // code cache doesn't need to be cleared as it's not dependent on block number

        // persist changes to DB
    }
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
            code_cache: Map::new(),
            account_memory_cache: Map::new(),
            account_cache: Map::new(),
            block_number_to_hash_cache: Map::new(),
            block_hash_to_number_cache: Map::new(),
            block_number_to_timestamp_cache: Map::new(),
            latest_block_hash: None,
        }
    }
}

impl DatabaseTrait for DB {
    type Error = Box<dyn Error>;

    /// Get basic account information.
    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        // println!("basic {}", address);
        let res = self.read_from_account_map(address)?;
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
        self.read_from_code_map(code_hash)
            .map(|x| x.unwrap_or(Bytecode::default()))
    }

    /// Get storage value of address at index.
    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        // println!("storage {} {}", address, index);
        self.read_from_account_memory_map(address, index)
            .map(|x| x.unwrap_or(U256::ZERO))
    }

    /// Get block hash by block number.
    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        // println!("block_hash {}", number);
        self.read_from_block_hashes(number)
            .map(|x| x.unwrap_or(B256::ZERO))
    }
}

impl DatabaseCommit for DB {
    fn commit(&mut self, changes: Map<Address, Account>) {
        for (address, account) in changes {
            if !account.is_touched() {
                continue;
            }
            if account.is_selfdestructed() {
                // TODO: check if working correctly!! (NOTE: wait for CANCUN update, after that selfdestruct will only work on newly created accounts)
                self.remove_from_account_map(address).unwrap();
                self.remove_from_account_memory_map(address).unwrap();
                continue;
            }
            let mut acc_info = AccountInfo::default();
            acc_info.balance = account.info.balance;
            acc_info.nonce = account.info.nonce;
            acc_info.code_hash = account.info.code_hash;
            self.set_account_map(address, acc_info).unwrap();

            let is_newly_created = account.is_created();
            if is_newly_created {
                // TODO: can contract change other than creation??
                let mut wtxn = self.get_write_txn().unwrap();
                self.set_code_map_with_txn(
                    account.info.code_hash,
                    account.info.code.unwrap(),
                    &mut wtxn,
                )
                .unwrap();
                wtxn.commit().unwrap();
            }

            for (loc, slot) in account.storage {
                if !slot.is_changed() {
                    continue;
                }
                self.set_account_memory_map(address, loc, slot.present_value())
                    .unwrap();
            }
        }
    }
}

fn get_lower_u256(val: U512ED) -> U256 {
    let limbs = val.0.as_limbs();
    let mut lower_limbs = [0u64; 4];
    for i in 0..4 {
        lower_limbs[i] = limbs[i];
    }

    Uint::from_limbs(lower_limbs)
}
