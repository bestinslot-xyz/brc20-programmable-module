use std::{borrow::Cow, error::Error};

use revm::primitives::ruint::aliases::U256;
use hashbrown::HashMap as Map;
use heed::{BytesDecode, BytesEncode};

use super::{as_eitem, from_ditem};

const MAX_HISTORY_SIZE: i32 = 10;

#[derive(Clone)]
pub struct BlockHistoryCacheData<'a, V>
where
    V: BytesEncode<'a> + BytesDecode<'a> + Clone,
{
    cache: Map<U256, Option<V>>,
    _phantom: std::marker::PhantomData<&'a V>,
}

pub trait BlockHistoryCache<'a, V>
where
    V: BytesEncode<'a> + BytesDecode<'a> + Clone,
{
    fn new() -> Self;
    fn latest(&self) -> Option<V>;
    fn get(&self, block_number: U256) -> Option<V>;
    fn set_empty(&mut self, block_number: U256);
    fn set(&mut self, block_number: U256, value: V);
    fn clear(&mut self);
}

impl<'a, V> BlockHistoryCache<'a, V> for BlockHistoryCacheData<'a, V>
where
    V: BytesEncode<'a> + BytesDecode<'a> + Clone,
{
    fn new() -> Self {
        Self { cache: Map::new(), _phantom: std::marker::PhantomData }
    }

    fn latest(&self) -> Option<V> {
        let result = self.cache.values().last();
        match result {
            Some(value) => value.clone(),
            None => None,
        }
    }

    fn get(&self, block_number: U256) -> Option<V> {
        let result = self.cache.get(&block_number);
        match result {
            Some(value) => value.clone(),
            None => None,
        }
    }

    fn set_empty(&mut self, block_number: U256) {
        self.cache.insert(block_number, None);
    }

    fn set(&mut self, block_number: U256, value: V) {
        self.cache.insert(block_number, Some(value));

        if self.cache.len() > (MAX_HISTORY_SIZE + 1) as usize {
            let mut keys = self.cache.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            let to_remove = keys[0].clone();
            self.cache.remove(&to_remove);
        }
    }

    fn clear(&mut self) {
        self.cache.clear();
    }
}

impl<'a, K> BytesEncode<'a> for BlockHistoryCacheData<'a, K>
where
    K: BytesEncode<'a> + BytesDecode<'a> + Clone + 'static,
{
    type EItem = BlockHistoryCacheData<'a, K>;

    fn bytes_encode(item: &'a Self::EItem) -> Result<Cow<'a, [u8]>, Box<dyn Error>> {
        let mut bytes = Vec::new();
        for (block_number, value) in item.cache.iter() {
            bytes.extend_from_slice(&U256::to_be_bytes::<32>(block_number));
            let value_eitem = as_eitem(value.as_ref().unwrap());
            let value_bytes = K::bytes_encode(value_eitem).unwrap();
            let size: u32 = value_bytes.len().try_into().unwrap();
            bytes.extend_from_slice(&size.to_be_bytes());
            bytes.extend_from_slice(&value_bytes);
        }
        Ok(Cow::Owned(bytes))
    }
}

impl<'a, K> BytesDecode<'a> for BlockHistoryCacheData<'a, K>
where 
    K: BytesEncode<'a> + BytesDecode<'a> + Clone + 'static,
{
    type DItem = BlockHistoryCacheData<'a, K>;

    fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem, Box<dyn Error>> {
        let mut cache: Map<U256, Option<K>> = Map::new();
        let mut i = 0;
        while i < bytes.len() {
            let block_number = U256::from_be_bytes::<32>(bytes[i..i + 32].try_into().unwrap());
            let size = u32::from_be_bytes(bytes[i + 32..i + 36].try_into().unwrap()) as usize;
            let value = K::bytes_decode(&bytes[i + 36..i + 36 + size]).unwrap();
            cache.insert(block_number, Some(from_ditem::<K>(&value).clone()));
            i += 36 + size; // 32 for block number + 4 for size + size for value
        }
        Ok(Self { cache, _phantom: std::marker::PhantomData })
    }
}

// Tests
#[cfg(test)]
mod tests {
    use heed::{BytesDecode, BytesEncode};
    use revm::primitives::ruint::aliases::U256;

    use super::{BlockHistoryCacheData, BlockHistoryCache};
    use crate::{cache::block_cache::MAX_HISTORY_SIZE, types::U256ED};

    #[test]
    fn test_block_cache() {
        let mut cache = BlockHistoryCacheData::<U256ED>::new();
        let block_number = U256::from(1);
        let value = U256::from(100);
        let value_ed = U256ED::from_u256(value);

        cache.set(block_number, value_ed.clone());
        assert_eq!(cache.get(block_number).unwrap().0, value_ed.0);
        assert_eq!(cache.latest().unwrap().0, value_ed.0);

        let value2 = U256::from(200);
        let value_ed2 = U256ED::from_u256(value2);
        cache.set(block_number, value_ed2.clone());
        assert_eq!(cache.get(block_number).unwrap().0, value_ed2.0);
        assert_eq!(cache.latest().unwrap().0, value_ed2.0);

        let block_number2 = U256::from(2);
        let value3 = U256::from(300);
        let value_ed3 = U256ED::from_u256(value3);
        cache.set(block_number2, value_ed3.clone());
        assert_eq!(cache.get(block_number2).unwrap().0, value_ed3.0);
        assert_eq!(cache.latest().unwrap().0, value_ed3.0);

        let value4 = U256::from(400);
        let value_ed4 = U256ED::from_u256(value4);
        cache.set(block_number2, value_ed4.clone());
        assert_eq!(cache.get(block_number2).unwrap().0, value_ed4.0);
        assert_eq!(cache.latest().unwrap().0, value_ed4.0);

        // Check old value still exists
        assert_eq!(cache.get(block_number).unwrap().0, value_ed2.0);

        cache.clear();
        assert!(cache.get(block_number).is_none());
        assert!(cache.get(block_number2).is_none());
        assert!(cache.latest().is_none());
    }

    #[test]
    fn test_history_size() {
        let mut cache = BlockHistoryCacheData::<U256ED>::new();
        let value = U256::from(100);
        let value_ed = U256ED::from_u256(value);

        for i in 0..MAX_HISTORY_SIZE + 2 {
            let block_number = U256::from(i);
            cache.set(block_number, value_ed.clone());
        }

        assert_eq!(cache.cache.len(), (MAX_HISTORY_SIZE + 1) as usize);
    }

    #[test]
    fn test_none_values() {
        let mut cache = BlockHistoryCacheData::<U256ED>::new();
        let block_number = U256::from(1);

        cache.set_empty(block_number);
        assert!(cache.get(block_number).is_none());
        assert!(cache.latest().is_none());
    }

    #[test]
    fn test_encode_decode() {
        let mut cache = BlockHistoryCacheData::<U256ED>::new();
        let block_number = U256::from(1);
        let value = U256::from(100);
        let value_ed = U256ED::from_u256(value);

        cache.set(block_number, value_ed.clone());
        let encoded = BlockHistoryCacheData::<U256ED>::bytes_encode(&cache).unwrap();
        let decoded = BlockHistoryCacheData::<U256ED>::bytes_decode(&encoded).unwrap();

        assert_eq!(decoded.get(block_number).unwrap().0, value_ed.0);
        assert_eq!(decoded.latest().unwrap().0, value_ed.0);
    }
}
