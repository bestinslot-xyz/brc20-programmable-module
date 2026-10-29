use std::collections::{BTreeMap, HashMap, HashSet};
use std::error::Error;
use std::hash::Hash;
use std::sync::Arc;

use rocksdb::{ColumnFamily, IteratorMode, WriteBatch, DB};

use crate::db::cached_database::BlockHistoryCache;
use crate::db::store::{cache_name, column_family};
use crate::db::types::{Decode, Encode};

// Database to store data that is mapped to a block number with a history cache
//
// It uses a cache to store the data in memory and only writes to the database when commit is called
// It also supports reorg by reverting back the state to the latest valid block
//
// The latest values live in the engine store's column family named after the database, and the
// histories in the `<name>_cache` column family.
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
    db: Arc<DB>,
    name: String,
    cache_name: String,
    cache: HashMap<K, C>,

    _phantom: std::marker::PhantomData<V>,
}

impl<K, V, C> BlockCachedDatabase<K, V, C>
where
    K: Encode + Decode + Eq + Hash + Clone,
    V: Encode + Decode + Eq + Clone,
    C: BlockHistoryCache<V> + Encode + Decode + Clone,
{
    /// Create a BlockCachedDatabase over the `name` and `<name>_cache` column families of the
    /// engine store
    ///
    /// db: Arc<DB> - the engine store
    /// name: &str - the name of the database
    ///
    /// Returns: BlockCachedDatabase<K, V, C> - the created BlockCachedDatabase
    pub fn new(db: Arc<DB>, name: &str) -> Result<Self, Box<dyn Error>> {
        let cache_name = cache_name(name);
        column_family(&db, name)?;
        column_family(&db, &cache_name)?;
        Ok(Self {
            db,
            name: name.to_string(),
            cache_name,
            cache: HashMap::new(),
            _phantom: std::marker::PhantomData,
        })
    }

    fn values_cf(&self) -> Result<&ColumnFamily, Box<dyn Error>> {
        column_family(&self.db, &self.name)
    }

    fn history_cf(&self) -> Result<&ColumnFamily, Box<dyn Error>> {
        column_family(&self.db, &self.cache_name)
    }

    /// Get the value for a key
    ///
    /// It first checks the cache and then the database
    /// If the value is not found, it returns None
    /// If the value is found, it returns Some(value)
    ///
    /// key: &K - the key to get the value for
    /// Returns: Option<V> - the value for the key
    pub fn latest(&self, key: &K) -> Result<Option<V>, Box<dyn Error>> {
        if let Some(cache) = self.cache.get(key) {
            return Ok(cache.latest());
        }
        if let Some(value) = self.db.get_cf(self.values_cf()?, key.encode_vec())? {
            let value = V::decode_vec(&value.to_vec())?;
            return Ok(Some(value));
        }
        return Ok(None);
    }

    /// Get the range of values between start_key and end_key
    ///
    /// It returns a list of key-value pairs between start_key and end_key, sorted by encoded key
    ///
    /// This only works if keys can be compared in their encoded form
    ///
    /// start_key: &K - the start key
    /// end_key: &K - the end key, exclusive
    /// Returns: Vec<(K, V)> - the list of key-value pairs, in ascending encoded key order
    pub fn get_range(&self, start_key: &K, end_key: &K) -> Result<Vec<(K, V)>, Box<dyn Error>> {
        let mut kv_pairs: BTreeMap<Vec<u8>, (K, V)> = BTreeMap::new();
        let start_key_bytes = start_key.encode_vec();
        let end_key_bytes = end_key.encode_vec();

        for kv_pair in self.db.iterator_cf(
            self.values_cf()?,
            IteratorMode::From(&start_key_bytes, rocksdb::Direction::Forward),
        ) {
            let (key_bytes, value) = kv_pair?;
            if *key_bytes >= *end_key_bytes {
                break;
            }
            let key = K::decode_vec(&key_bytes.to_vec())?;
            let value = V::decode_vec(&value.to_vec())?;
            kv_pairs.insert(key_bytes.to_vec(), (key, value));
        }

        for (key, cache) in &self.cache {
            let key_bytes = key.encode_vec();
            if *key_bytes < *start_key_bytes || *key_bytes >= *end_key_bytes {
                continue;
            }
            if let Some(value) = cache.latest() {
                kv_pairs.insert(key_bytes, (key.clone(), value));
            } else {
                kv_pairs.remove(&key_bytes);
            }
        }

        Ok(kv_pairs.into_values().collect())
    }

    /// Returns all keys and values in the database
    ///
    /// It returns a list of all key-value pairs in the database
    pub fn all(&self) -> Result<Vec<(K, V)>, Box<dyn Error>> {
        let mut kv_pairs: HashMap<K, V> = HashMap::new();

        for kv_pair in self
            .db
            .full_iterator_cf(self.values_cf()?, IteratorMode::Start)
        {
            let (key, value) = kv_pair?;
            let key = K::decode_vec(&key.to_vec())?;
            let value = V::decode_vec(&value.to_vec())?;
            if !kv_pairs.contains_key(&key) {
                kv_pairs.insert(key, value);
            }
        }

        for (key, cache) in &self.cache {
            if let Some(value) = cache.latest() {
                kv_pairs.insert(key.clone(), value.clone());
            } else {
                kv_pairs.remove(key);
            }
        }

        Ok(kv_pairs.into_iter().collect())
    }

    /// Set the value for a key
    ///
    /// It sets the value in the cache, it's not written to the database until commit is called
    ///
    /// block_number: U256 - the block number to set the value for
    /// key: K - the key to set the value for
    /// value: V - the value to set
    pub fn set(&mut self, block_number: u64, key: &K, value: V) -> Result<(), Box<dyn Error>> {
        let cache = self.retrieve_cache(&key)?;
        cache.set(block_number, value);
        Ok(())
    }

    /// Unset the value for a key
    ///
    /// It removes the value from the cache, it's not written to the database until commit is called
    /// block_number: U256 - the block number to unset the value for
    /// key: K - the key to unset the value for
    pub fn unset(&mut self, block_number: u64, key: &K) -> Result<(), Box<dyn Error>> {
        let cache = self.retrieve_cache(&key)?;
        cache.unset(block_number);
        Ok(())
    }

    /// Add the cache to a commit's write batch
    ///
    /// For every cached key, the batch stores its latest value and its history, and drops a
    /// history too old for a reorg. The values reach the database when the batch is written.
    /// The cache is kept; clear it once the batch is written.
    ///
    /// block_number: U256 - the block number to commit at
    pub fn stage_commit(
        &self,
        batch: &mut WriteBatch,
        block_number: u64,
    ) -> Result<(), Box<dyn Error>> {
        let values_cf = self.values_cf()?;
        let history_cf = self.history_cf()?;
        for (key, cache) in self.cache.iter() {
            let key_bytes = key.encode_vec();
            if cache.is_old(block_number) {
                batch.delete_cf(history_cf, &key_bytes);
            } else {
                batch.put_cf(history_cf, &key_bytes, cache.encode_vec());
            }

            if let Some(value) = cache.latest() {
                batch.put_cf(values_cf, &key_bytes, value.encode_vec());
            } else {
                batch.delete_cf(values_cf, &key_bytes);
            }
        }
        Ok(())
    }

    /// Revert the state to the latest valid block
    ///
    /// It reverts every history to the latest valid block, adds the reverted state to the write
    /// batch and clears the cache
    ///
    /// latest_valid_block_number: U256 - the latest valid block number
    pub fn stage_reorg(
        &mut self,
        batch: &mut WriteBatch,
        latest_valid_block_number: u64,
    ) -> Result<(), Box<dyn Error>> {
        let mut keys = HashSet::new();
        {
            for kv_pair in self
                .db
                .full_iterator_cf(self.history_cf()?, IteratorMode::Start)
            {
                keys.insert(K::decode_vec(&kv_pair?.0.to_vec())?);
            }
            for key in self.cache.keys() {
                keys.insert(key.clone());
            }
        }
        for key in keys {
            let cache = self.retrieve_cache(&key)?;
            cache.reorg(latest_valid_block_number);
        }
        self.stage_commit(batch, latest_valid_block_number)?;
        self.clear_cache();
        Ok(())
    }

    /// Write the cache to the database in its own batch and clear the cache
    #[cfg(test)]
    pub fn commit(&mut self, block_number: u64) -> Result<(), Box<dyn Error>> {
        let mut batch = WriteBatch::default();
        self.stage_commit(&mut batch, block_number)?;
        self.db.write(batch)?;
        self.clear_cache();
        Ok(())
    }

    /// Revert the state to the latest valid block in its own batch
    #[cfg(test)]
    pub fn reorg(&mut self, latest_valid_block_number: u64) -> Result<(), Box<dyn Error>> {
        let mut batch = WriteBatch::default();
        self.stage_reorg(&mut batch, latest_valid_block_number)?;
        self.db.write(batch)?;
        Ok(())
    }

    /// Clear the cache
    ///
    /// It clears the cache, make sure to call commit before clearing the cache to write the data to the database
    /// Otherwise the data will be lost
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    fn retrieve_cache(&mut self, key: &K) -> Result<&mut C, Box<dyn Error>> {
        if self.cache.contains_key(key) {
            // Do nothing, the cache is already in memory
        } else if let Some(cache_bytes) = self.db.get_cf(self.history_cf()?, key.encode_vec())? {
            let cache = C::decode_vec(&cache_bytes.to_vec())?;
            self.cache.insert(key.clone(), cache);
        } else {
            // This is a cache miss, retrieve the value from the database to make sure
            // we have an old value at hand if reorg occurs
            let stored_value = self
                .db
                .get_cf(self.values_cf()?, key.encode_vec())?
                .and_then(|value| V::decode_vec(&value.to_vec()).ok());
            self.cache.insert(key.clone(), C::new(stored_value));
        }
        Ok(self.cache.get_mut(key).ok_or("Cache not found")?)
    }
}

#[cfg(test)]
mod tests {
    use alloy::primitives::{Address, B256, U256};
    use revm::state::AccountInfo;
    use tempfile::TempDir;

    use super::*;
    use crate::db::cached_database::BlockHistoryCacheData;
    use crate::db::store;
    use crate::db::types::{AccountInfoED, AddressED, B256ED, U128ED};

    #[test]
    fn test_cache_only() {
        let path = TempDir::new().unwrap();
        let mut db = BlockCachedDatabase::<
            AddressED,
            AccountInfoED,
            BlockHistoryCacheData<AccountInfoED>,
        >::new(store::open(path.path()).unwrap(), "account")
        .unwrap();

        let address: Address = "0x1234567890123456789012345678901234567890"
            .parse()
            .unwrap();
        let account_info: AccountInfoED = AccountInfo {
            balance: U256::from(100),
            nonce: 1,
            code_hash: [1; 32].into(),
            code: None,
            ..Default::default()
        }
        .into();
        let address_ed: AddressED = address.into();
        let _ = db.set(1, &address_ed.clone(), account_info.clone());

        let account_info = db.latest(&address_ed).unwrap().unwrap();
        assert_eq!(account_info.balance, 100u64.into());
        assert_eq!(account_info.nonce, 1u64.into());
        assert_eq!(account_info.code_hash, [1; 32].into());

        // verify the cache content
        let cache = db.cache.get(&address_ed).unwrap();
        assert_eq!(cache.latest().unwrap().balance, U256::from(100).into());
        assert_eq!(cache.latest().unwrap().nonce, 1u64.into());
        assert_eq!(cache.latest().unwrap().code_hash, [1; 32].into());
    }

    #[test]
    fn test_get_range_returns_key_order() {
        let path = TempDir::new().unwrap();
        let mut db = BlockCachedDatabase::<U128ED, B256ED, BlockHistoryCacheData<B256ED>>::new(
            store::open(path.path()).unwrap(),
            "number_and_index_to_tx_hash",
        )
        .unwrap();
        let key = |block: u64, index: u64| U128ED::from(((block as u128) << 64) | index as u128);
        let value = |index: u64| B256ED::from(B256::from(U256::from(index)));

        // Committed entries for block 1, inserted out of order
        for index in [7, 2, 9, 0, 4] {
            db.set(1, &key(1, index), value(index)).unwrap();
        }
        db.commit(1).unwrap();

        // Cached entries for block 1, plus one for block 2 outside the range
        for index in [5, 1, 8, 3, 6] {
            db.set(2, &key(1, index), value(index)).unwrap();
        }
        db.set(2, &key(2, 0), value(100)).unwrap();
        db.unset(2, &key(1, 9)).unwrap();

        let range = db.get_range(&key(1, 0), &key(2, 0)).unwrap();
        let expected: Vec<(U128ED, B256ED)> =
            (0..9).map(|index| (key(1, index), value(index))).collect();
        assert_eq!(range, expected);
    }

    #[test]
    fn test_database_commit() {
        let path = TempDir::new().unwrap();
        let mut db = BlockCachedDatabase::<
            AddressED,
            AccountInfoED,
            BlockHistoryCacheData<AccountInfoED>,
        >::new(store::open(path.path()).unwrap(), "account")
        .unwrap();

        let address: Address = "0x1234567890123456789012345678901234567890"
            .parse()
            .unwrap();
        let address_ed: AddressED = address.into();
        let account_info_ed: AccountInfoED = AccountInfo {
            balance: U256::from(100),
            nonce: 1,
            code_hash: [1; 32].into(),
            code: None,
            ..Default::default()
        }
        .into();
        let _ = db.set(1, &address.into(), account_info_ed.clone());

        let account_info = db.latest(&address.into()).unwrap().unwrap();
        assert_eq!(account_info.balance, U256::from(100).into());
        assert_eq!(account_info.nonce, 1u64.into());
        assert_eq!(account_info.code_hash, B256::from([1; 32]).into());

        db.commit(1).unwrap();

        let account_info = db
            .db
            .get_cf(db.values_cf().unwrap(), address_ed.encode_vec())
            .unwrap();

        let account_info = AccountInfoED::decode_vec(&account_info.unwrap().to_vec()).unwrap();
        assert_eq!(account_info.balance, U256::from(100).into());
        assert_eq!(account_info.nonce, 1u64.into());
        assert_eq!(account_info.code_hash, B256::from([1; 32]).into());

        let cache = db
            .db
            .get_cf(db.history_cf().unwrap(), address_ed.encode_vec())
            .unwrap();

        let cache =
            BlockHistoryCacheData::<AccountInfoED>::decode_vec(&cache.unwrap().to_vec()).unwrap();
        assert_eq!(cache.latest().unwrap().balance, U256::from(100).into());
        assert_eq!(cache.latest().unwrap().nonce, 1u64.into());
        assert_eq!(
            cache.latest().unwrap().code_hash,
            B256::from([1; 32]).into()
        );
    }

    #[test]
    fn test_database_reorg() {
        let path = TempDir::new().unwrap();
        let mut db = BlockCachedDatabase::<
            AddressED,
            AccountInfoED,
            BlockHistoryCacheData<AccountInfoED>,
        >::new(store::open(path.path()).unwrap(), "account")
        .unwrap();

        let address: Address = "0x1234567890123456789012345678901234567890"
            .parse()
            .unwrap();
        let account_info: AccountInfoED = AccountInfo {
            balance: U256::from(100),
            nonce: 1,
            code_hash: [1; 32].into(),
            code: None,
            ..Default::default()
        }
        .into();
        let address_ed: AddressED = address.into();
        let _ = db.set(1, &address_ed.clone(), account_info.clone());

        let account_info = db.latest(&address_ed).unwrap().unwrap();
        assert_eq!(account_info.balance, U256::from(100).into());
        assert_eq!(account_info.nonce, 1u64.into());
        assert_eq!(account_info.code_hash, B256::from([1; 32]).into());

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
        >::new(store::open(path.path()).unwrap(), "account")
        .unwrap();

        let address: Address = "0x1234567890123456789012345678901234567890"
            .parse()
            .unwrap();
        let address_ed: AddressED = address.into();

        for i in 1..=10 {
            let _ = db.set(
                i,
                &address_ed.clone(),
                AccountInfo {
                    balance: U256::from(100 + i),
                    nonce: i + 1,
                    code_hash: [1; 32].into(),
                    code: None,
                    ..Default::default()
                }
                .into(),
            );
        }
        db.commit(10).unwrap();

        db.reorg(5).unwrap();

        db.commit(5).unwrap();

        let account_info = db.latest(&address_ed).unwrap().unwrap();
        assert_eq!(account_info.balance, U256::from(100 + 5).into());
        assert_eq!(account_info.nonce, 6u64.into());
        assert_eq!(account_info.code_hash, B256::from([1; 32]).into());
    }

    #[test]
    fn test_reorg_after_removing_an_old_cache() {
        let path = TempDir::new().unwrap();
        let mut db = BlockCachedDatabase::<
            AddressED,
            AccountInfoED,
            BlockHistoryCacheData<AccountInfoED>,
        >::new(store::open(path.path()).unwrap(), "account")
        .unwrap();

        let address: Address = "0x1234567890123456789012345678901234567890"
            .parse()
            .unwrap();
        let address_ed: AddressED = address.into();

        db.set(
            10,
            &address_ed.clone(),
            AccountInfo {
                balance: U256::from(100 + 10),
                nonce: 10,
                code_hash: [1; 32].into(),
                code: None,
                ..Default::default()
            }
            .into(),
        )
        .unwrap();

        // Value committed at block height 10 first
        db.commit(10).unwrap();

        // This should remove the old cache, as the cache latest value is at block height 10
        db.commit(22).unwrap();
        db.reorg(21).unwrap();

        // Value set at block height 22, causing a new cache to be created
        db.set(
            22,
            &address_ed.clone(),
            AccountInfo {
                balance: U256::from(100 + 21),
                nonce: 22,
                code_hash: [1; 32].into(),
                code: None,
                ..Default::default()
            }
            .into(),
        )
        .unwrap();

        // Another reorg at block height 22, with an old cache
        // that doesn't see beyond block 11, this fails.
        db.commit(22).unwrap();
        db.reorg(21).unwrap();

        let account_info = db.latest(&address_ed).unwrap().unwrap();
        assert_eq!(account_info.balance, U256::from(100 + 10).into());
        assert_eq!(account_info.nonce, 10u64.into());
        assert_eq!(account_info.code_hash, B256::from([1; 32]).into());
    }
}
