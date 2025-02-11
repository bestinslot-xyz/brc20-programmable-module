use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use heed::{Database, Env, Result, RwTxn};

use crate::{
    cached_database::BytesWrapper,
    types::{Decode, Encode},
};

use super::BlockHistoryCache;

// Database to store data that is mapped to a block number with a history cache
//
// It uses a cache to store the data in memory and only writes to the database when commit is called
// It also supports reorg by reverting back the state to the latest valid block
//
// K: the type of the key
// V: the type of the value to store
// C: the type of the cache
// It should implement BlockHistoryCache<V> to store the history of the value
pub struct BlockCachedDatabase<K, V, C>
where
    K: Encode + Decode + Eq + Hash + Clone,
    V: Encode + Decode + Clone + Eq,
    C: BlockHistoryCache<V> + Encode + Decode + Clone,
{
    env: Env,
    db: Database<BytesWrapper, BytesWrapper>,
    cache_db: Database<BytesWrapper, BytesWrapper>,
    cache: HashMap<K, C>,

    _phantom: std::marker::PhantomData<V>,
}

impl<K, V, C> BlockCachedDatabase<K, V, C>
where
    K: Encode + Decode + Eq + Hash + Clone,
    V: Encode + Decode + Eq + Clone,
    C: BlockHistoryCache<V> + Encode + Decode + Clone,
{
    /// Create a new BlockCachedDatabase
    ///
    /// It creates a new database if it does not exist
    ///
    /// env: heed::Env - the environment to store the database
    /// name: &str - the name of the database
    /// parent_wtxn: &mut heed::RwTxn<'_, '_> - the write transaction to use
    ///
    /// Returns: BlockCachedDatabase<K, V, C> - the created BlockCachedDatabase
    pub fn new(env: Env, name: &str, parent_wtxn: &mut RwTxn) -> Self {
        let db: Database<BytesWrapper, BytesWrapper> = {
            let old_db = env
                .open_database::<BytesWrapper, BytesWrapper>(&parent_wtxn, Some(name))
                .unwrap();
            if old_db.is_some() {
                old_db.unwrap()
            } else {
                env.create_database(parent_wtxn, Some(name)).unwrap()
            }
        };
        let cache_db = {
            let cache_name = format!("{}_cache", &name);
            let old_db = env
                .open_database::<BytesWrapper, BytesWrapper>(&parent_wtxn, Some(&cache_name))
                .unwrap();
            if old_db.is_some() {
                old_db.unwrap()
            } else {
                env.create_database(parent_wtxn, Some(&cache_name)).unwrap()
            }
        };
        let cache = HashMap::new();
        Self {
            env: env.clone(),
            db,
            cache_db,
            cache,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get the value for a key
    ///
    /// It first checks the cache and then the database
    /// If the value is not found, it returns None
    /// If the value is found, it returns Some(value)
    ///
    /// key: &K - the key to get the value for
    /// Returns: Option<V> - the value for the key
    pub fn latest(&self, key: &K) -> Option<V> {
        if self.cache.contains_key(key) {
            let cache = self.cache.get(key).unwrap();
            return cache.latest();
        }
        let rtxn = self.env.read_txn().unwrap();
        let result = self
            .db
            .get(&rtxn, &BytesWrapper::from_vec(K::encode(key).unwrap()));
        if result.is_err() {
            return None;
        }
        let value = result.unwrap();
        if value.is_none() {
            return None;
        }
        Some(V::decode(value.unwrap().to_vec()).unwrap())
    }

    /// Set the value for a key
    ///
    /// It sets the value in the cache, it's not written to the database until commit is called
    ///
    /// block_number: U256 - the block number to set the value for
    /// key: K - the key to set the value for
    /// value: V - the value to set
    pub fn set(&mut self, block_number: u64, key: K, value: V) -> Result<()> {
        if self.cache.contains_key(&key) {
            let cache = self.cache.get_mut(&key).unwrap();
            cache.set(block_number, value);
        } else {
            self.load_cache_if_needed(&key)?;
            let cache = self.cache.get_mut(&key).unwrap();
            cache.set(block_number, value);
        }
        Ok(())
    }

    /// Commit the cache to the database
    ///
    /// It writes all the values in the cache to the database
    /// It does not clear the cache
    ///
    /// parent_wtxn: &mut heed::RwTxn<'_, '_> - the write transaction to use
    pub fn commit(&self, mut parent_wtxn: &mut RwTxn, block_number: u64) -> Result<()> {
        for (key, cache) in self.cache.iter() {
            let key_bytes = BytesWrapper::from_vec(K::encode(key).unwrap());
            let cache_bytes = BytesWrapper::from_vec(C::encode(cache).unwrap());
            // Delete old caches from the cache_db
            if cache.is_old(block_number) {
                self.cache_db.delete(&mut parent_wtxn, &key_bytes)?;
            } else {
                self.cache_db
                    .put(&mut parent_wtxn, &key_bytes, &cache_bytes)?;
            }

            if cache.latest().is_none() {
                self.db.delete(&mut parent_wtxn, &key_bytes)?;
            } else {
                self.db.put(
                    &mut parent_wtxn,
                    &key_bytes,
                    &BytesWrapper::from_vec(V::encode(&cache.latest().unwrap()).unwrap()),
                )?;
            }
        }
        Ok(())
    }

    /// Revert the state to the latest valid block
    ///
    /// It reverts the state of all the caches to the latest valid block
    /// It does not clear the cache
    ///
    /// parent_wtxn: &mut heed::RwTxn<'_, '_> - the write transaction to use
    /// latest_valid_block_number: U256 - the latest valid block number
    pub fn reorg(
        &mut self,
        mut parent_wtxn: &mut RwTxn,
        latest_valid_block_number: u64,
    ) -> Result<()> {
        let mut keys = HashSet::new();
        {
            let rtxn = self.env.read_txn()?;
            for result in self.cache_db.iter(&rtxn).unwrap() {
                let (key, _) = result?;
                keys.insert(K::decode(key.to_vec()).unwrap());
            }
            for key in self.cache.keys() {
                keys.insert(key.clone());
            }
        }
        for key in keys {
            self.load_cache_if_needed(&key)?;
            let cache = self.cache.get_mut(&key).unwrap();
            cache.reorg(latest_valid_block_number);
        }
        self.commit(&mut parent_wtxn, latest_valid_block_number)?;
        Ok(())
    }

    /// Clear the cache
    ///
    /// It clears the cache, make sure to call commit before clearing the cache to write the data to the database
    /// Otherwise the data will be lost
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    fn load_cache_if_needed(&mut self, key: &K) -> Result<()> {
        if self.cache.contains_key(key) {
            return Ok(());
        }

        let rtxn = self.env.read_txn()?;
        let cache_bytes = self
            .cache_db
            .get(&rtxn, &BytesWrapper::from_vec(K::encode(key).unwrap()));
        if cache_bytes.is_err() {
            let latest_db_value = self
                .db
                .get(&rtxn, &BytesWrapper::from_vec(K::encode(key).unwrap()));
            if latest_db_value.is_err() {
                self.cache.insert(key.clone(), C::new(None));
            } else {
                let initial_cache_value: Option<V> = latest_db_value
                    .unwrap()
                    .map(|v| V::decode(v.to_vec()).unwrap());
                self.cache.insert(key.clone(), C::new(initial_cache_value));
            }
        } else {
            let value = cache_bytes.unwrap();
            if value.is_none() {
                self.cache.insert(key.clone(), C::new(None));
                return Ok(());
            }
            let cache = C::decode(value.unwrap().to_vec()).unwrap();
            self.cache.insert(key.clone(), cache);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use revm::primitives::AccountInfo;
    use revm::primitives::Address;
    use revm::primitives::B256;
    use revm::primitives::U256;

    use crate::cached_database::BytesWrapper;
    use crate::cached_database::{BlockCachedDatabase, BlockHistoryCache, BlockHistoryCacheData};
    use crate::test_utils::db::create_test_env;
    use crate::types::Decode;
    use crate::types::Encode;
    use crate::types::{AccountInfoED, AddressED};

    #[test]
    fn test_cache_only() {
        let wrapper = create_test_env();
        let env = &wrapper.env;
        let mut wtxn = env.write_txn().unwrap();
        let mut db = BlockCachedDatabase::<
            AddressED,
            AccountInfoED,
            BlockHistoryCacheData<AccountInfoED>,
        >::new(env.clone(), "test_db", &mut wtxn);
        wtxn.commit().unwrap();

        let address: Address = "0x1234567890123456789012345678901234567890"
            .parse()
            .unwrap();
        let account_info = AccountInfoED::from_account_info(AccountInfo {
            balance: U256::from(100),
            nonce: 1,
            code_hash: B256::from([1; 32]),
            code: None,
        });
        let address_ed = AddressED::from_addr(address);
        let _ = db.set(1, address_ed.clone(), account_info.clone());

        let account_info = db.latest(&address_ed).unwrap();
        assert_eq!(account_info.0.balance, U256::from(100));
        assert_eq!(account_info.0.nonce, 1);
        assert_eq!(account_info.0.code_hash, B256::from([1; 32]));

        // verify the cache content
        let cache = db.cache.get(&address_ed).unwrap();
        assert_eq!(cache.latest().unwrap().0.balance, U256::from(100));
        assert_eq!(cache.latest().unwrap().0.nonce, 1);
        assert_eq!(cache.latest().unwrap().0.code_hash, B256::from([1; 32]));
    }

    #[test]
    fn test_database_commit() {
        let wrapper = create_test_env();
        let env = &wrapper.env;
        let mut wtxn = env.write_txn().unwrap();
        let mut db = BlockCachedDatabase::<
            AddressED,
            AccountInfoED,
            BlockHistoryCacheData<AccountInfoED>,
        >::new(env.clone(), "test_db", &mut wtxn);
        wtxn.commit().unwrap();

        let address: Address = "0x1234567890123456789012345678901234567890"
            .parse()
            .unwrap();
        let account_info = AccountInfoED::from_account_info(AccountInfo {
            balance: U256::from(100),
            nonce: 1,
            code_hash: B256::from([1; 32]),
            code: None,
        });
        let address_ed = AddressED::from_addr(address);
        let _ = db.set(1, address_ed.clone(), account_info.clone());

        let account_info = db.latest(&address_ed).unwrap();
        assert_eq!(account_info.0.balance, U256::from(100));
        assert_eq!(account_info.0.nonce, 1);
        assert_eq!(account_info.0.code_hash, B256::from([1; 32]));

        let mut wtxn = env.write_txn().unwrap();
        db.commit(&mut wtxn, 1).unwrap();
        wtxn.commit().unwrap();

        let real_db = db.db;

        let rtxn = env.read_txn().unwrap();
        let account_info = real_db
            .get(
                &rtxn,
                &BytesWrapper::from_vec(AddressED::encode(&address_ed).unwrap()),
            )
            .unwrap();

        let account_info = AccountInfoED::decode(account_info.unwrap().to_vec()).unwrap();
        assert_eq!(account_info.0.balance, U256::from(100));
        assert_eq!(account_info.0.nonce, 1);
        assert_eq!(account_info.0.code_hash, B256::from([1; 32]));

        let cache_db = db.cache_db;

        let cache = cache_db
            .get(
                &rtxn,
                &BytesWrapper::from_vec(AddressED::encode(&address_ed).unwrap()),
            )
            .unwrap();

        let cache =
            BlockHistoryCacheData::<AccountInfoED>::decode(cache.unwrap().to_vec()).unwrap();
        assert_eq!(cache.latest().unwrap().0.balance, U256::from(100));
        assert_eq!(cache.latest().unwrap().0.nonce, 1);
        assert_eq!(cache.latest().unwrap().0.code_hash, B256::from([1; 32]));
    }

    #[test]
    fn test_database_reorg() {
        let wrapper = create_test_env();
        let env = &wrapper.env;
        let mut wtxn = env.write_txn().unwrap();
        let mut db = BlockCachedDatabase::<
            AddressED,
            AccountInfoED,
            BlockHistoryCacheData<AccountInfoED>,
        >::new(env.clone(), "test_db", &mut wtxn);
        wtxn.commit().unwrap();

        let address: Address = "0x1234567890123456789012345678901234567890"
            .parse()
            .unwrap();
        let account_info = AccountInfoED::from_account_info(AccountInfo {
            balance: U256::from(100),
            nonce: 1,
            code_hash: B256::from([1; 32]),
            code: None,
        });
        let address_ed = AddressED::from_addr(address);
        let _ = db.set(1, address_ed.clone(), account_info.clone());

        let account_info = db.latest(&address_ed).unwrap();
        assert_eq!(account_info.0.balance, U256::from(100));
        assert_eq!(account_info.0.nonce, 1);
        assert_eq!(account_info.0.code_hash, B256::from([1; 32]));

        let mut wtxn = env.write_txn().unwrap();
        db.commit(&mut wtxn, 1).unwrap();
        wtxn.commit().unwrap();

        let mut wtxn = env.write_txn().unwrap();
        db.reorg(&mut wtxn, 0).unwrap();
        wtxn.commit().unwrap();

        db.clear_cache();

        let account_info = db.latest(&address_ed);
        assert!(account_info.is_none());
    }

    #[test]
    fn test_database_reorg_10_blocks() {
        let wrapper = create_test_env();
        let env = &wrapper.env;
        let mut wtxn = env.write_txn().unwrap();
        let mut db = BlockCachedDatabase::<
            AddressED,
            AccountInfoED,
            BlockHistoryCacheData<AccountInfoED>,
        >::new(env.clone(), "test_db", &mut wtxn);
        wtxn.commit().unwrap();

        let address: Address = "0x1234567890123456789012345678901234567890"
            .parse()
            .unwrap();
        let address_ed = AddressED::from_addr(address);

        for i in 1..=10 {
            let _ = db.set(
                i,
                address_ed.clone(),
                AccountInfoED::from_account_info(AccountInfo {
                    balance: U256::from(100 + i),
                    nonce: i + 1,
                    code_hash: B256::from([1; 32]),
                    code: None,
                }),
            );
        }
        let mut wtxn = env.write_txn().unwrap();
        db.commit(&mut wtxn, 10).unwrap();
        wtxn.commit().unwrap();

        let mut wtxn = env.write_txn().unwrap();
        db.reorg(&mut wtxn, 5).unwrap();
        wtxn.commit().unwrap();

        let mut wtxn = env.write_txn().unwrap();
        db.commit(&mut wtxn, 5).unwrap();
        wtxn.commit().unwrap();

        let account_info = db.latest(&address_ed).unwrap();
        assert_eq!(account_info.0.balance, U256::from(100 + 5));
        assert_eq!(account_info.0.nonce, 1 + 5);
        assert_eq!(account_info.0.code_hash, B256::from([1; 32]));
    }
}
