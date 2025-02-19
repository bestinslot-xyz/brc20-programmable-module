use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    path::Path,
};

use rocksdb::{Error, IteratorMode, Options, DB};

use crate::types::{Decode, Encode};

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
    db: DB,
    cache_db: DB,
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
    /// path: &Path - the path to store the database
    /// name: &str - the name of the database
    ///
    /// Returns: BlockCachedDatabase<K, V, C> - the created BlockCachedDatabase
    pub fn new(path: &Path, name: &str) -> Self {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, &path.join(Path::new(name))).unwrap();
        let cache_db = DB::open(&opts, &path.join(Path::new(&format!("{}_cache", name)))).unwrap();
        let cache = HashMap::new();
        Self {
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
    pub fn latest(&self, key: &K) -> Result<Option<V>, Error> {
        if self.cache.contains_key(key) {
            let cache = self.cache.get(key).unwrap();
            return Ok(cache.latest());
        }
        let result = self.db.get(key.encode().unwrap())?;
        if result.is_none() {
            return Ok(None);
        }
        let value = V::decode(result.unwrap().to_vec()).unwrap();
        Ok(Some(value))
    }

    /// Get the range of values between start_key and end_key
    ///
    /// It returns a list of key-value pairs between start_key and end_key
    ///
    /// This only works if keys can be compared in their encoded form
    ///
    /// start_key: &K - the start key
    /// end_key: &K - the end key
    /// Returns: Vec<(K, V)> - the list of key-value pairs
    pub fn get_range(&self, start_key: &K, end_key: &K) -> Result<Vec<(K, V)>, Error> {
        let mut result = Vec::new();
        let start_key_bytes = start_key.encode().unwrap();
        let end_key_bytes = end_key.encode().unwrap();

        for key in self.cache.keys() {
            let key_bytes = key.encode().unwrap();
            if *key_bytes < *start_key_bytes {
                continue;
            }
            if *key_bytes >= *end_key_bytes {
                break;
            }
            let cache = self.cache.get(key).unwrap();
            if cache.latest().is_none() {
                continue;
            }
            result.push((key.clone(), cache.latest().unwrap()));
        }

        for kv_pair in self.db.iterator(IteratorMode::From(
            &start_key_bytes,
            rocksdb::Direction::Forward,
        )) {
            #[cfg(debug_assertions)]
            println!("{:?}", kv_pair);

            let (key, value) = kv_pair?;
            if *key > *end_key_bytes {
                break;
            }
            let key = K::decode(key.to_vec()).unwrap();
            let value = V::decode(value.to_vec()).unwrap();
            result.push((key, value));
        }

        Ok(result)
    }

    /// Set the value for a key
    ///
    /// It sets the value in the cache, it's not written to the database until commit is called
    ///
    /// block_number: U256 - the block number to set the value for
    /// key: K - the key to set the value for
    /// value: V - the value to set
    pub fn set(&mut self, block_number: u64, key: K, value: V) -> Result<(), Error> {
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
    /// block_number: U256 - the block number to commit at
    pub fn commit(&mut self, block_number: u64) -> Result<(), Error> {
        for (key, cache) in self.cache.iter() {
            let key_bytes = K::encode(key).unwrap();
            let cache_bytes = C::encode(cache).unwrap();
            if cache.is_old(block_number) {
                self.cache_db.delete(&key_bytes)?;
            } else {
                self.cache_db.put(&key_bytes, &cache_bytes)?;
            }

            if cache.latest().is_none() {
                self.db.delete(&key_bytes)?;
            } else {
                self.db
                    .put(&key_bytes, &cache.latest().unwrap().encode().unwrap())?;
            }
        }

        let keys_to_remove: Vec<K> = self
            .cache
            .iter()
            .filter(|(_, cache)| cache.is_old(block_number))
            .map(|(key, _)| key.clone())
            .collect();
        for key in keys_to_remove {
            self.cache.remove(&key);
        }

        self.clear_cache();
        Ok(())
    }

    /// Revert the state to the latest valid block
    ///
    /// It reverts the state of all the caches to the latest valid block
    /// It does not clear the cache
    ///
    /// latest_valid_block_number: U256 - the latest valid block number
    pub fn reorg(&mut self, latest_valid_block_number: u64) -> Result<(), Error> {
        let mut keys = HashSet::new();
        {
            for kv_pair in self.cache_db.full_iterator(IteratorMode::Start) {
                keys.insert(K::decode(kv_pair.unwrap().0.to_vec()).unwrap());
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
        self.commit(latest_valid_block_number)?;
        self.clear_cache();
        Ok(())
    }

    /// Clear the cache
    ///
    /// It clears the cache, make sure to call commit before clearing the cache to write the data to the database
    /// Otherwise the data will be lost
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    fn load_cache_if_needed(&mut self, key: &K) -> Result<(), Error> {
        if self.cache.contains_key(key) {
            return Ok(());
        }

        let cache_bytes = self.cache_db.get(key.encode().unwrap())?;
        if cache_bytes.is_none() {
            self.cache.insert(key.clone(), C::new(None));
            return Ok(());
        }
        let cache = C::decode(cache_bytes.unwrap().to_vec()).unwrap();
        self.cache.insert(key.clone(), cache);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use revm::primitives::AccountInfo;
    use revm::primitives::Address;
    use revm::primitives::B256;
    use revm::primitives::U256;
    use tempfile::TempDir;

    use crate::cached_database::{BlockCachedDatabase, BlockHistoryCache, BlockHistoryCacheData};
    use crate::types::Decode;
    use crate::types::Encode;
    use crate::types::{AccountInfoED, AddressED};

    #[test]
    fn test_cache_only() {
        let path = TempDir::new().unwrap();
        let mut db = BlockCachedDatabase::<
            AddressED,
            AccountInfoED,
            BlockHistoryCacheData<AccountInfoED>,
        >::new(path.path(), "test_db");

        let address: Address = "0x1234567890123456789012345678901234567890"
            .parse()
            .unwrap();
        let account_info = AccountInfoED(AccountInfo {
            balance: U256::from(100),
            nonce: 1,
            code_hash: B256::from([1; 32]),
            code: None,
        });
        let address_ed = AddressED(address);
        let _ = db.set(1, address_ed.clone(), account_info.clone());

        let account_info = db.latest(&address_ed).unwrap().unwrap();
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
        let path = TempDir::new().unwrap();
        let mut db = BlockCachedDatabase::<
            AddressED,
            AccountInfoED,
            BlockHistoryCacheData<AccountInfoED>,
        >::new(path.path(), "test_db");

        let address: Address = "0x1234567890123456789012345678901234567890"
            .parse()
            .unwrap();
        let account_info = AccountInfoED(AccountInfo {
            balance: U256::from(100),
            nonce: 1,
            code_hash: B256::from([1; 32]),
            code: None,
        });
        let address_ed = AddressED(address);
        let _ = db.set(1, address_ed.clone(), account_info.clone());

        let account_info = db.latest(&address_ed).unwrap().unwrap();
        assert_eq!(account_info.0.balance, U256::from(100));
        assert_eq!(account_info.0.nonce, 1);
        assert_eq!(account_info.0.code_hash, B256::from([1; 32]));

        db.commit(1).unwrap();

        let real_db = db.db;

        let account_info = real_db.get(address_ed.encode().unwrap()).unwrap();

        let account_info = AccountInfoED::decode(account_info.unwrap().to_vec()).unwrap();
        assert_eq!(account_info.0.balance, U256::from(100));
        assert_eq!(account_info.0.nonce, 1);
        assert_eq!(account_info.0.code_hash, B256::from([1; 32]));

        let cache_db = db.cache_db;

        let cache = cache_db.get(address_ed.encode().unwrap()).unwrap();

        let cache =
            BlockHistoryCacheData::<AccountInfoED>::decode(cache.unwrap().to_vec()).unwrap();
        assert_eq!(cache.latest().unwrap().0.balance, U256::from(100));
        assert_eq!(cache.latest().unwrap().0.nonce, 1);
        assert_eq!(cache.latest().unwrap().0.code_hash, B256::from([1; 32]));
    }

    #[test]
    fn test_database_reorg() {
        let path = TempDir::new().unwrap();
        let mut db = BlockCachedDatabase::<
            AddressED,
            AccountInfoED,
            BlockHistoryCacheData<AccountInfoED>,
        >::new(path.path(), "test_db");

        let address: Address = "0x1234567890123456789012345678901234567890"
            .parse()
            .unwrap();
        let account_info = AccountInfoED(AccountInfo {
            balance: U256::from(100),
            nonce: 1,
            code_hash: B256::from([1; 32]),
            code: None,
        });
        let address_ed = AddressED(address);
        let _ = db.set(1, address_ed.clone(), account_info.clone());

        let account_info = db.latest(&address_ed).unwrap().unwrap();
        assert_eq!(account_info.0.balance, U256::from(100));
        assert_eq!(account_info.0.nonce, 1);
        assert_eq!(account_info.0.code_hash, B256::from([1; 32]));

        db.commit(1).unwrap();
        db.reorg(0).unwrap();
        db.clear_cache();

        let account_info = db.latest(&address_ed);
        assert!(account_info.unwrap().is_none());
    }

    #[test]
    fn test_database_reorg_10_blocks() {
        let path = TempDir::new().unwrap();
        let mut db = BlockCachedDatabase::<
            AddressED,
            AccountInfoED,
            BlockHistoryCacheData<AccountInfoED>,
        >::new(path.path(), "test_db");

        let address: Address = "0x1234567890123456789012345678901234567890"
            .parse()
            .unwrap();
        let address_ed = AddressED(address);

        for i in 1..=10 {
            let _ = db.set(
                i,
                address_ed.clone(),
                AccountInfoED(AccountInfo {
                    balance: U256::from(100 + i),
                    nonce: i + 1,
                    code_hash: B256::from([1; 32]),
                    code: None,
                }),
            );
        }
        db.commit(10).unwrap();

        db.reorg(5).unwrap();

        db.commit(5).unwrap();

        let account_info = db.latest(&address_ed).unwrap().unwrap();
        assert_eq!(account_info.0.balance, U256::from(100 + 5));
        assert_eq!(account_info.0.nonce, 1 + 5);
        assert_eq!(account_info.0.code_hash, B256::from([1; 32]));
    }
}
