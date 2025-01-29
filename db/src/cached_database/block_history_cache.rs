use std::{collections::BTreeMap, error::Error};

use crate::types::{Decode, Encode};

const MAX_HISTORY_SIZE: u64 = 10;

// Cache to store the history of a value at different block numbers
#[derive(Clone)]
pub struct BlockHistoryCacheData<V>
where
    V: Encode + Decode + Clone,
{
    cache: BTreeMap<u64, Option<V>>,
}

pub trait BlockHistoryCache<V>
where
    V: Encode + Decode + Clone,
{
    fn new(initial_value: Option<V>) -> Self;
    fn latest(&self) -> Option<V>;
    fn set(&mut self, block_number: u64, value: V);
    fn reorg(&mut self, latest_valid_block_number: u64);
}

impl<V> BlockHistoryCache<V> for BlockHistoryCacheData<V>
where
    V: Encode + Decode + Clone,
{
    // Create a new BlockHistoryCache
    //
    // initial_value: Option<V> - the initial value to store
    //
    // Returns: BlockHistoryCacheData<V> - the created BlockHistoryCache
    fn new(initial_value: Option<V>) -> Self {
        let mut cache = BTreeMap::new();
        if let Some(value) = initial_value {
            cache.insert(0, Some(value));
        } else {
            cache.insert(0, None);
        }
        Self { cache }
    }

    // Get the latest value
    fn latest(&self) -> Option<V> {
        self.cache.values().last().cloned().unwrap_or(None)
    }

    // Set the value for a block number
    //
    // block_number: U256 - the block number
    // value: V - the value to store
    fn set(&mut self, block_number: u64, value: V) {
        self.cache.insert(block_number, Some(value));

        // Remove the oldest value if the cache size is greater than MAX_HISTORY_SIZE + 1
        // The extra 1 is to keep the initial value
        if self.cache.len() > (MAX_HISTORY_SIZE + 1) as usize {
            let keys_to_remove: Vec<u64> = self
                .cache
                .keys()
                .take(self.cache.len() - (MAX_HISTORY_SIZE + 1) as usize)
                .cloned()
                .collect();
            for key in keys_to_remove {
                self.cache.remove(&key);
            }
        }
    }

    // Reorganize the cache, removing all values with block number greater than the latest valid block number
    //
    // latest_valid_block_number: U256 - the latest valid block number
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
}

impl<V> Encode for BlockHistoryCacheData<V>
where
    V: Encode + Decode + Clone,
{
    fn encode(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut bytes = Vec::new();
        for (block_number, value) in self.cache.iter() {
            bytes.extend_from_slice(&block_number.to_be_bytes());
            if value.is_none() {
                bytes.extend_from_slice(&0u32.to_be_bytes());
                continue;
            }
            let value_bytes = value.as_ref().unwrap().encode().unwrap();
            let size: u32 = value_bytes.len().try_into().unwrap();
            bytes.extend_from_slice(&size.to_be_bytes());
            bytes.extend_from_slice(&value_bytes);
        }
        Ok(bytes)
    }
}

impl<V> Decode for BlockHistoryCacheData<V>
where
    V: Encode + Decode + Clone,
{
    fn decode(bytes: Vec<u8>) -> Result<Self, Box<dyn Error>> {
        let mut cache: BTreeMap<u64, Option<V>> = BTreeMap::new();
        let mut i = 0;
        while i < bytes.len() {
            let block_number = u64::from_be_bytes(bytes[i..i + 32].try_into().unwrap());
            i += 32;
            let size = u32::from_be_bytes(bytes[i..i + 4].try_into().unwrap()) as usize;
            i += 4;
            if size == 0 {
                cache.insert(block_number, None);
                continue;
            }
            let value = V::decode((&bytes[i..i + size]).to_vec()).unwrap();
            cache.insert(block_number, Some(value));
            i += size;
        }
        Ok(Self { cache })
    }
}

// Tests
#[cfg(test)]
mod tests {
    use revm::primitives::ruint::aliases::U256;

    use super::{BlockHistoryCache, BlockHistoryCacheData};
    use crate::{
        cached_database::block_history_cache::MAX_HISTORY_SIZE,
        types::{Decode, Encode, U256ED},
    };

    #[test]
    fn test_block_cache() {
        let mut cache = BlockHistoryCacheData::<U256ED>::new(None);
        let block_number = 1;
        let value = U256::from(100);
        let value_ed = U256ED::from_u256(value);

        cache.set(block_number, value_ed.clone());
        assert_eq!(cache.latest().unwrap().0, value_ed.0);

        let value2 = U256::from(200);
        let value_ed2 = U256ED::from_u256(value2);
        cache.set(block_number, value_ed2.clone());
        assert_eq!(cache.latest().unwrap().0, value_ed2.0);

        let block_number2 = 2;
        let value3 = U256::from(300);
        let value_ed3 = U256ED::from_u256(value3);
        cache.set(block_number2, value_ed3.clone());
        assert_eq!(cache.latest().unwrap().0, value_ed3.0);

        let value4 = U256::from(400);
        let value_ed4 = U256ED::from_u256(value4);
        cache.set(block_number2, value_ed4.clone());
        assert_eq!(cache.latest().unwrap().0, value_ed4.0);
    }

    #[test]
    fn test_history_size() {
        let mut cache = BlockHistoryCacheData::<U256ED>::new(None);
        let value = U256::from(100);
        let value_ed = U256ED::from_u256(value);

        for i in 0..MAX_HISTORY_SIZE + 2 {
            cache.set(i, value_ed.clone());
        }

        assert_eq!(cache.cache.len(), (MAX_HISTORY_SIZE + 1) as usize);
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
        let value_ed = U256ED::from_u256(value);

        cache.set(block_number, value_ed.clone());
        let encoded = BlockHistoryCacheData::<U256ED>::encode(&cache).unwrap();
        let decoded = BlockHistoryCacheData::<U256ED>::decode(encoded).unwrap();

        assert_eq!(decoded.latest().unwrap().0, value_ed.0);
    }

    #[test]
    fn test_reorg() {
        let mut cache = BlockHistoryCacheData::<U256ED>::new(None);

        let block_number = 1;
        let value = U256::from(100);
        let value_ed = U256ED::from_u256(value);
        cache.set(block_number, value_ed.clone());
        assert_eq!(cache.latest().unwrap().0, value_ed.0);

        let block_number2 = 2;
        let value2 = U256::from(200);
        let value_ed2 = U256ED::from_u256(value2);
        cache.set(block_number2, value_ed2.clone());
        assert_eq!(cache.latest().unwrap().0, value_ed2.0);

        cache.reorg(1);

        assert_eq!(cache.latest().unwrap().0, value_ed.0);
    }

    #[test]
    fn test_reorg_multiple_blocks() {
        let mut cache = BlockHistoryCacheData::<U256ED>::new(None);

        for i in 1..=11 {
            let value = U256::from(100 * i);
            let value_ed = U256ED::from_u256(value);
            cache.set(i, value_ed.clone());
            assert_eq!(cache.latest().unwrap().0, value_ed.0);
        }

        cache.reorg(5);

        assert_eq!(cache.latest().unwrap().0, U256::from(500));
    }

    #[test]
    fn test_reorg_all_blocks() {
        let mut cache = BlockHistoryCacheData::<U256ED>::new(None);

        for i in 1..=11 {
            let value = U256::from(100 * i);
            let value_ed = U256ED::from_u256(value);
            cache.set(i, value_ed.clone());
            assert_eq!(cache.latest().unwrap().0, value_ed.0);
        }

        cache.reorg(0);

        assert!(cache.latest().is_none());
    }
}
