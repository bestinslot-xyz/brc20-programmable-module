use std::collections::BTreeMap;
use std::error::Error;

use crate::db::types::{Decode, Encode};
use crate::global::MAX_REORG_HISTORY_SIZE;

/// Cache to store the history of a value at different block numbers
#[derive(Clone)]
pub struct BlockHistoryCacheData<V>
where
    V: Encode + Decode + Clone + Eq,
{
    cache: BTreeMap<u64, Option<V>>,
}

pub trait BlockHistoryCache<V>
where
    V: Encode + Decode + Clone + Eq,
{
    fn new(initial_value: Option<V>) -> Self;
    fn latest(&self) -> Option<V>;
    fn set(&mut self, block_number: u64, value: V);
    fn unset(&mut self, block_number: u64);
    fn reorg(&mut self, latest_valid_block_number: u64);
    fn is_old(&self, block_number: u64) -> bool;
}

impl<V> BlockHistoryCacheData<V>
where
    V: Encode + Decode + Clone + Eq,
{
    fn remove_old_values(&mut self, latest_block_number: u64) {
        // Remove the old values
        // Any key that is less than the set block number - MAX_HISTORY_SIZE is too old
        let keys_to_remove: Vec<u64> = self
            .cache
            .keys()
            .filter(|&&key| key + MAX_REORG_HISTORY_SIZE <= latest_block_number)
            .cloned()
            .collect();

        // Remove all except the last one to keep at least one value in the cache
        if keys_to_remove.len() != 0 {
            for key in keys_to_remove.iter().take(keys_to_remove.len() - 1) {
                self.cache.remove(key);
            }
        }
    }
}

impl<V> BlockHistoryCache<V> for BlockHistoryCacheData<V>
where
    V: Encode + Decode + Clone + Eq,
{
    /// Create a new BlockHistoryCache
    ///
    /// initial_value: Option<V> - the initial value to store
    ///
    /// Returns: BlockHistoryCacheData<V> - the created BlockHistoryCache
    fn new(initial_value: Option<V>) -> Self {
        let mut cache = BTreeMap::new();
        cache.insert(0, initial_value);
        Self { cache }
    }

    /// Get the latest value
    fn latest(&self) -> Option<V> {
        self.cache.values().last().cloned().unwrap_or(None)
    }

    /// Set the value for a block number
    ///
    /// block_number: U256 - the block number
    /// value: V - the value to store
    fn set(&mut self, block_number: u64, value: V) {
        // Only allow block numbers greater than the latest block number to avoid tampering with the history
        if let Some((latest_stored_block_number, _)) = self.cache.iter().last() {
            if block_number < *latest_stored_block_number {
                panic!("Block number must be greater than or equal to the latest block number");
            }
        }
        // This is to avoid storing the same value multiple times
        if let Some(latest) = self.latest() {
            if latest == value {
                return;
            }
        }
        self.cache.insert(block_number, Some(value));

        // Remove old values to keep the cache size within MAX_REORG_HISTORY_SIZE
        self.remove_old_values(block_number);
    }

    fn unset(&mut self, block_number: u64) {
        // Set the value to None for the given block number
        if let Some((latest_stored_block_number, _)) = self.cache.iter().last() {
            if block_number < *latest_stored_block_number {
                panic!("Block number must be greater than or equal to the latest block number");
            }
        }
        // This is to avoid storing the same value multiple times
        if self.latest().is_none() {
            return;
        }
        self.cache.insert(block_number, None);

        // Remove old values to keep the cache size within MAX_REORG_HISTORY_SIZE
        self.remove_old_values(block_number);
    }

    /// Reorganize the cache, removing all values with block number greater than the latest valid block number
    ///
    /// latest_valid_block_number: U256 - the latest valid block number
    fn reorg(&mut self, latest_valid_block_number: u64) {
        let keys_to_remove: Vec<u64> = self
            .cache
            .keys()
            .filter(|&&key| key > latest_valid_block_number)
            .cloned()
            .collect();
        for key in keys_to_remove {
            self.cache.remove(&key);
        }
    }

    /// Check if the cache is too old for a reorg at the latest block number
    ///
    /// A cache is considered too old for a reorg if the latest stored value is older than MAX_HISTORY_SIZE
    ///
    /// Caches too old for a reorg should be removed from the database, as they are not useful anymore
    ///
    /// block_number: u64 - current block number
    /// Returns: bool - true if the cache is empty, false otherwise
    fn is_old(&self, latest_block_number: u64) -> bool {
        if let Some(&latest_stored_block_number) = self.cache.keys().last() {
            return latest_stored_block_number + MAX_REORG_HISTORY_SIZE < latest_block_number;
        } else {
            return true;
        }
    }
}

impl<V> Encode for BlockHistoryCacheData<V>
where
    V: Encode + Decode + Clone + Eq,
{
    fn encode(&self, buffer: &mut Vec<u8>) {
        (self.cache.len() as u32).encode(buffer);
        for (block_number, value) in &self.cache {
            block_number.encode(buffer);
            value.encode(buffer);
        }
    }
}

impl<V> Decode for BlockHistoryCacheData<V>
where
    V: Encode + Decode + Clone + Eq,
{
    fn decode(bytes: &[u8], offset: usize) -> Result<(Self, usize), Box<dyn Error>> {
        let mut cache: BTreeMap<u64, Option<V>> = BTreeMap::new();
        let (len, mut offset) = u32::decode(bytes, offset)?;
        for _ in 0..len {
            let (block_number, offset_temp) = Decode::decode(bytes, offset)?;
            offset = offset_temp;
            let (value, offset_temp) = Decode::decode(bytes, offset)?;
            offset = offset_temp;
            cache.insert(block_number, value);
        }
        Ok((Self { cache }, offset))
    }
}

// Tests
#[cfg(test)]
mod tests {
    use alloy::primitives::U256;

    use super::*;
    use crate::db::types::U256ED;

    #[test]
    fn test_block_cache() {
        let mut cache = BlockHistoryCacheData::<U256ED>::new(None);
        let block_number = 1;
        let value_ed: U256ED = U256::from(100).into();

        cache.set(block_number, value_ed.clone());
        assert_eq!(cache.latest().unwrap(), value_ed);

        let value_ed2: U256ED = U256::from(200).into();
        cache.set(block_number, value_ed2.clone());
        assert_eq!(cache.latest().unwrap(), value_ed2);

        let block_number2 = 2;
        let value_ed3: U256ED = U256::from(300).into();
        cache.set(block_number2, value_ed3.clone());
        assert_eq!(cache.latest().unwrap(), value_ed3);

        let value_ed4: U256ED = U256::from(400).into();
        cache.set(block_number2, value_ed4.clone());
        assert_eq!(cache.latest().unwrap(), value_ed4);
    }

    #[test]
    fn test_history_size() {
        let mut cache = BlockHistoryCacheData::<U256ED>::new(None);

        for i in 0..MAX_REORG_HISTORY_SIZE + 2 {
            let value = U256::from(100 * i);
            cache.set(i, value.into());
        }

        assert_eq!(cache.cache.len(), (MAX_REORG_HISTORY_SIZE + 1) as usize);
    }

    #[test]
    fn test_none_values() {
        let cache = BlockHistoryCacheData::<U256ED>::new(None);

        assert!(cache.latest().is_none());
    }

    #[test]
    fn test_encode_decode() {
        let mut cache = BlockHistoryCacheData::<U256ED>::new(None);
        let block_number = 1;
        let value = U256::from(100);

        cache.set(block_number, value.into());
        let encoded = cache.encode_vec();
        let decoded = BlockHistoryCacheData::<U256ED>::decode_vec(&encoded).unwrap();

        assert_eq!(decoded.latest().unwrap().uint, value);
    }

    #[test]
    fn test_reorg() {
        let mut cache = BlockHistoryCacheData::<U256ED>::new(None);

        let block_number = 1;
        let value = U256::from(100);
        cache.set(block_number, value.into());
        assert_eq!(cache.latest().unwrap(), value.into());

        let block_number2 = 2;
        let value2 = U256::from(200);
        cache.set(block_number2, value2.into());
        assert_eq!(cache.latest().unwrap(), value2.into());

        cache.reorg(1);

        assert_eq!(cache.latest().unwrap(), value.into());
    }

    #[test]
    fn test_reorg_multiple_blocks() {
        let mut cache = BlockHistoryCacheData::<U256ED>::new(None);

        for i in 1..=11 {
            let value = U256::from(100 * i);
            cache.set(i, value.into());
            assert_eq!(cache.latest().unwrap(), value.into());
        }

        cache.reorg(5);

        assert_eq!(cache.latest().unwrap(), U256::from(500).into());
    }

    #[test]
    fn test_reorg_all_blocks() {
        let mut cache = BlockHistoryCacheData::<U256ED>::new(None);

        for i in 1..=11 {
            let value = U256::from(100 * i);
            cache.set(i, value.into());
            assert_eq!(cache.latest().unwrap(), value.into());
        }

        cache.reorg(0);

        assert!(cache.latest().is_none());
    }

    #[test]
    fn test_same_values() {
        let mut cache = BlockHistoryCacheData::<U256ED>::new(None);

        let value = U256::from(100);
        cache.set(0, value.into());
        assert_eq!(cache.latest().unwrap(), value.into());

        cache.set(1, value.into());
        assert_eq!(cache.latest().unwrap(), value.into());

        cache.set(2, value.into());

        assert_eq!(cache.cache.len(), 1);
    }
}
