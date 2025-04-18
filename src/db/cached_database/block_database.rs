use std::collections::BTreeMap;
use std::error::Error;
use std::path::Path;

use rocksdb::{IteratorMode, Options, DB};

use crate::db::types::{Decode, Encode, U64ED};

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
    db: DB,
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
    /// path: &Path - the path to the database
    /// name: &str - the name of the database
    ///
    /// Returns: BlockDatabase<V> - the created BlockDatabase
    pub fn new(path: &Path, name: &str) -> Result<Self, Box<dyn Error>> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_max_open_files(256);
        let db = DB::open(&opts, &path.join(Path::new(name)))?;
        Ok(Self {
            db,
            cache: BTreeMap::new(),
        })
    }

    /// Get the value for a block number
    //
    /// It first checks the cache and then the database
    /// If the value is not found, it returns None
    /// If the value is found, it returns Some(value)
    //
    /// block_number: u64 - the block number to get the value for
    /// Returns: Option<V> - the value for the block number
    pub fn get(&self, key: u64) -> Result<Option<V>, Box<dyn Error>> {
        if let Some(value) = self.cache.get(&key) {
            return Ok(Some(value.clone()));
        }

        let Some(value_bytes) = self.db.get(U64ED::from(key).encode())? else {
            return Ok(None);
        };

        let value = V::decode(value_bytes)?;
        Ok(Some(value))
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
    pub fn commit(&mut self) -> Result<(), Box<dyn Error>> {
        for (key, value) in self.cache.iter() {
            let value_bytes = value.encode();
            self.db.put(U64ED::from(*key).encode(), &value_bytes)?;
        }
        self.db.flush()?;
        Ok(())
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
    pub fn last_key(&self) -> Result<Option<u64>, Box<dyn Error>> {
        let db_last_key = match self.db.full_iterator(IteratorMode::End).take(1).last() {
            Some(Ok((key, _))) => Some(U64ED::decode(key.to_vec())?.into()),
            _ => None,
        };

        let cache_last_key = self.cache.keys().last().map(|key| *key);

        Ok(std::cmp::max(db_last_key, cache_last_key))
    }

    /// Reorg the database
    //
    /// It deletes all the data that is not valid anymore, i.e. the data with block number greater than latest_valid_block_number
    /// Make sure to call commit after calling this function to write the changes to the database
    //
    /// latest_valid_block_number: u64 - the latest valid block number
    pub fn reorg(&mut self, latest_valid_block_number: u64) -> Result<(), Box<dyn Error>> {
        let mut current = latest_valid_block_number + 1;
        let last_block = self.last_key()?;
        if let Some(end) = last_block {
            while end >= current {
                self.db.delete(U64ED::from(current).encode())?;
                self.cache.remove(&current);
                current += 1;
            }
        }
        Ok(())
    }
}

// tests
#[cfg(test)]
mod tests {
    use alloy_primitives::U256;
    use tempfile::TempDir;

    use super::*;
    use crate::db::types::U256ED;

    #[test]
    fn test_block_database() {
        let tempdir = TempDir::new().unwrap();
        let mut db = BlockDatabase::<U256ED>::new(tempdir.path(), "test").unwrap();

        let block_number = 1;
        let value = U256::from(100).into();
        db.set(block_number, value);

        let block_number = 2;
        let value = U256::from(200).into();
        db.set(block_number, value);

        let block_number = 3;
        let value = U256::from(300).into();
        db.set(block_number, value);

        assert_eq!(db.last_key().unwrap().unwrap(), 3);

        db.commit().unwrap();
        db.clear_cache();

        assert_eq!(db.get(1).unwrap().unwrap(), U256::from(100).into());
        assert_eq!(db.get(2).unwrap().unwrap(), U256::from(200).into());
        assert_eq!(db.get(3).unwrap().unwrap(), U256::from(300).into());
        assert_eq!(db.last_key().unwrap().unwrap(), 3);

        db.reorg(2).unwrap();

        assert_eq!(db.get(1).unwrap().unwrap(), U256::from(100).into());
        assert_eq!(db.get(2).unwrap().unwrap(), U256::from(200).into());
        assert_eq!(db.get(3).unwrap().is_none(), true);

        assert_eq!(db.last_key().unwrap().unwrap(), 2);
    }
}
