use std::collections::BTreeMap;

use heed::{Database, Env, RwTxn};

use crate::types::{Decode, Encode, U64ED};

use super::BytesWrapper;

/// Database to store data that is mapped to a block number
///
/// It uses a cache to store the data in memory and only writes to the database when commit is called
/// It also supports reorg by deleting data that is not valid anymore
/// It uses U256 as the key (block_number) and V as the value
///
/// V: the type of the value to store
/// It should implement Encode, Decode and Clone, so that it can be serialized and deserialized
pub struct BlockDatabase<V>
where
    V: Encode + Decode + Clone,
{
    env: Env,
    db: Database<BytesWrapper, BytesWrapper>,
    cache: BTreeMap<u64, V>,
}

impl<V> BlockDatabase<V>
where
    V: Encode + Decode + Clone,
{
    /// Create a new BlockDatabase
    ///
    /// It creates a new database if it does not exist
    ///
    /// env: heed::Env - the environment to store the database
    /// name: &str - the name of the database
    /// parent_wtxn: &mut heed::RwTxn<'_, '_> - the write transaction to use
    ///
    /// Returns: BlockDatabase<V> - the created BlockDatabase
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
        Self {
            env,
            db,
            cache: BTreeMap::new(),
        }
    }

    /// Get the value for a block number
    //
    /// It first checks the cache and then the database
    /// If the value is not found, it returns None
    /// If the value is found, it returns Some(value)
    //
    /// block_number: u64 - the block number to get the value for
    /// Returns: Option<V> - the value for the block number
    pub fn get(&mut self, key: u64) -> Option<V> {
        if let Some(value) = self.cache.get(&key) {
            return Some(value.clone());
        }

        let rtxn = self.env.read_txn().unwrap();
        let value_bytes = self
            .db
            .get(
                &rtxn,
                &BytesWrapper::from_vec(U64ED::from_u64(key).encode().unwrap()),
            )
            .unwrap();
        if value_bytes.is_none() {
            return None;
        }
        let value = V::decode(value_bytes.unwrap().to_vec()).unwrap();
        self.cache.insert(key.clone(), value.clone());
        Some(value)
    }

    /// Set the value for a block number
    //
    /// It sets the value in the cache, it's not written to the database until commit is called
    //
    /// block_number: u64 - the block number to set the value for
    /// value: V - the value to set
    pub fn set(&mut self, block_number: u64, value: V) {
        self.cache.insert(block_number, value.clone());
    }

    /// Commit the cache to the database
    //
    /// It writes all the values in the cache to the database
    /// It does not clear the cache
    //
    /// parent_wtxn: &mut heed::RwTxn<'_, '_> - the write transaction to use
    pub fn commit(&mut self, mut parent_wtxn: &mut RwTxn) {
        for (key, value) in self.cache.iter() {
            let value_bytes = BytesWrapper::from_vec(value.encode().unwrap());
            self.db
                .put(
                    &mut parent_wtxn,
                    &BytesWrapper::from_vec(U64ED::from_u64(*key).encode().unwrap()),
                    &value_bytes,
                )
                .unwrap();
        }
    }

    /// Clear the cache
    //
    /// It clears the cache
    //
    /// This does not delete the data from the database, make sure to call commit before clearing the cache
    /// to write the data to the database, otherwise the data will be lost
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get the last key in the database
    //
    /// It returns the last key in the database
    /// If the database is empty, it returns None
    //
    /// Returns: Option<u64> - the last key in the database
    pub fn last_key(&self) -> Option<u64> {
        let rtxn = self.env.read_txn().unwrap();
        let result = self
            .db
            .last(&rtxn)
            .unwrap()
            .map(|(key, _)| U64ED::decode(key.to_vec()).unwrap().to_u64());

        // if cache has larger value, replace result
        if let Some((key, _)) = self.cache.iter().last() {
            if result.is_none() || key > result.as_ref().unwrap() {
                return Some(key.clone());
            } else {
                return result;
            }
        } else {
            return result;
        }
    }

    /// Reorg the database
    //
    /// It deletes all the data that is not valid anymore, i.e. the data with block number greater than latest_valid_block_number
    /// Make sure to call commit on parent_wtxn after calling this function to write the changes to the database
    //
    // parent_wtxn: &mut heed::RwTxn<'_, '_> - the write transaction to use
    pub fn reorg(&mut self, parent_wtxn: &mut RwTxn, latest_valid_block_number: u64) {
        let mut current = latest_valid_block_number + 1;
        let end = self.last_key().unwrap();
        while end >= current {
            self.db
                .delete(
                    parent_wtxn,
                    &BytesWrapper::from_vec(U64ED::from_u64(current).encode().unwrap()),
                )
                .unwrap();
            self.cache.remove(&end);
            current += 1;
        }
    }
}

// tests
#[cfg(test)]
mod tests {
    use revm::primitives::U256;

    use crate::{cached_database::BlockDatabase, test_utils::db::create_test_env, types::U256ED};

    #[test]
    fn test_block_database() {
        let env_wrapper = create_test_env();
        let env = &env_wrapper.env;
        let mut wtxn = env.write_txn().unwrap();
        let mut db = BlockDatabase::<U256ED>::new(env.clone(), "test", &mut wtxn);
        wtxn.commit().unwrap();

        let block_number = 1;
        let value = U256ED::from_u256(U256::from(100));
        db.set(block_number, value);

        let block_number = 2;
        let value = U256ED::from_u256(U256::from(200));
        db.set(block_number, value);

        let block_number = 3;
        let value = U256ED::from_u256(U256::from(300));
        db.set(block_number, value);

        assert_eq!(db.last_key().unwrap(), 3);

        let mut wtxn = env.write_txn().unwrap();
        db.commit(&mut wtxn);
        wtxn.commit().unwrap();
        db.clear_cache();

        assert_eq!(db.get(1).unwrap(), U256ED::from_u256(U256::from(100)));
        assert_eq!(db.get(2).unwrap(), U256ED::from_u256(U256::from(200)));
        assert_eq!(db.get(3).unwrap(), U256ED::from_u256(U256::from(300)));
        assert_eq!(db.last_key(), Some(3));

        let mut wtxn = env.write_txn().unwrap();
        db.reorg(&mut wtxn, 2);
        wtxn.commit().unwrap();

        assert_eq!(db.get(1).unwrap(), U256ED::from_u256(U256::from(100)));
        assert_eq!(db.get(2).unwrap(), U256ED::from_u256(U256::from(200)));
        assert_eq!(db.get(3).is_none(), true);

        assert_eq!(db.last_key().unwrap(), 2);
    }
}
