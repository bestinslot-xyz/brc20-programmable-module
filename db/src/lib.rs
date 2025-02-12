use std::{collections::HashMap, error::Error, path::Path};

use cached_database::BlockDatabase;

use revm::primitives::{
    alloy_primitives::{U128, U64},
    db::{Database as DatabaseTrait, DatabaseCommit},
    ruint::aliases::U256,
    Account, AccountInfo, Address, Bytecode, B256,
};

mod cached_database;
use cached_database::{BlockCachedDatabase, BlockHistoryCacheData};

mod types;
use types::{AccountInfoED, AddressED, BytecodeED, B256ED, U128ED, U256ED, U512ED, U64ED};

pub struct DB {
    /// Account address to memory location
    /// TODO: If the value is zero, consider deleting it from the database to save space
    db_account_memory: Option<BlockCachedDatabase<U512ED, U256ED, BlockHistoryCacheData<U256ED>>>,

    /// Code hash to bytecode
    db_code: Option<BlockCachedDatabase<B256ED, BytecodeED, BlockHistoryCacheData<BytecodeED>>>,

    /// Account address to account info
    db_account:
        Option<BlockCachedDatabase<AddressED, AccountInfoED, BlockHistoryCacheData<AccountInfoED>>>,

    /// Block hash to block number
    db_block_hash_to_number:
        Option<BlockCachedDatabase<B256ED, U64ED, BlockHistoryCacheData<U64ED>>>,

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
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let base_path = Path::new("target").join("db");
        let db_account_memory = Some(BlockCachedDatabase::new(&base_path, "account_memory_map"));
        let db_code = Some(BlockCachedDatabase::new(&base_path, "code_map"));
        let db_account = Some(BlockCachedDatabase::new(&base_path, "account_map"));
        let db_block_hash_to_number =
            Some(BlockCachedDatabase::new(&base_path, "block_hash_to_number"));

        let db_block_number_to_hash = Some(BlockDatabase::new(&base_path, "block_hash"));
        let db_block_number_to_timestamp = Some(BlockDatabase::new(&base_path, "block_ts"));
        let db_block_number_to_gas_used = Some(BlockDatabase::new(&base_path, "gas_used"));
        let db_block_number_to_mine_tm = Some(BlockDatabase::new(&base_path, "mine_tm"));

        Ok(Self {
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

    pub fn get_latest_block_height(&self) -> Result<u64, Box<dyn Error>> {
        if self.latest_block_number.is_some() {
            return Ok(self.latest_block_number.unwrap().0);
        }
        Ok(self
            .db_block_number_to_hash
            .as_ref()
            .unwrap()
            .last_key()?
            .unwrap_or(0))
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
            .latest(&U512ED::from_addr_u256(account, mem_loc))?;

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
            .latest(&B256ED::from_b256(code_hash))?;

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
            .latest(&AddressED::from_addr(account))?;

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

    pub fn get_block_number(&mut self, block_hash: B256) -> Result<Option<u64>, Box<dyn Error>> {
        let ret = self
            .db_block_hash_to_number
            .as_ref()
            .unwrap()
            .latest(&B256ED::from_b256(block_hash))?;

        Ok(ret.map(|x| x.to_u64()))
    }

    pub fn get_block_hash(&mut self, block_number: u64) -> Result<Option<B256>, Box<dyn Error>> {
        let ret = self
            .db_block_number_to_hash
            .as_mut()
            .unwrap()
            .get(block_number)?;

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
            .get(number)?;

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
            .get(block_number)?;

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
            .get(block_number)?;

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
        let latest_block_number = self.get_latest_block_height()?;

        self.db_block_number_to_hash.as_mut().unwrap().commit()?;
        self.db_block_number_to_timestamp
            .as_mut()
            .unwrap()
            .commit()?;
        self.db_block_number_to_gas_used
            .as_mut()
            .unwrap()
            .commit()?;
        self.db_block_number_to_mine_tm.as_mut().unwrap().commit()?;

        self.db_account_memory
            .as_mut()
            .unwrap()
            .commit(latest_block_number)?;
        self.db_code.as_mut().unwrap().commit(latest_block_number)?;
        self.db_account
            .as_mut()
            .unwrap()
            .commit(latest_block_number)?;
        self.db_block_hash_to_number
            .as_mut()
            .unwrap()
            .commit(latest_block_number)?;

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
        self.db_account_memory
            .as_mut()
            .unwrap()
            .reorg(latest_valid_block_number)?;
        self.db_code
            .as_mut()
            .unwrap()
            .reorg(latest_valid_block_number)?;
        self.db_account
            .as_mut()
            .unwrap()
            .reorg(latest_valid_block_number)?;
        self.db_block_hash_to_number
            .as_mut()
            .unwrap()
            .reorg(latest_valid_block_number)?;

        self.db_block_number_to_hash
            .as_mut()
            .unwrap()
            .reorg(latest_valid_block_number)?;
        self.db_block_number_to_timestamp
            .as_mut()
            .unwrap()
            .reorg(latest_valid_block_number)?;
        self.db_block_number_to_gas_used
            .as_mut()
            .unwrap()
            .reorg(latest_valid_block_number)?;
        self.db_block_number_to_mine_tm
            .as_mut()
            .unwrap()
            .reorg(latest_valid_block_number)?;

        self.commit_changes()?;
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
