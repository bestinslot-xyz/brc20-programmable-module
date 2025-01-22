use std::hash::Hash;

use hashbrown::HashMap as Map;
use heed::{BytesDecode, BytesEncode, Database, Env, Result, RoTxn, RwTxn};
use revm::primitives::U256;

use super::{as_ditem, as_eitem, from_ditem, from_eitem, BlockHistoryCache};

// K is the key type
// V is the value type
// C is the cache type which should be a BlockCache type with V as the value type
pub struct BlockCachedDatabase<'a, K, V, C>
where
    K: BytesEncode<'a> + BytesDecode<'a> + Eq + Hash + Clone,
    V: BytesEncode<'a> + BytesDecode<'a> + Clone,
    C: BlockHistoryCache<'a, V> + BytesDecode<'a> + BytesEncode<'a> + Clone,
{
    env: &'a Env,
    rtxn: RoTxn<'a>,
    db: heed::Database<K, V>,
    cache_db: Database<K, C>,
    cache: Map<K, C>,

    _phantom: std::marker::PhantomData<&'a V>,
}

impl<'a, K, V, C> BlockCachedDatabase<'a, K, V, C>
where
    K: BytesEncode<'a> + BytesDecode<'a> + Eq + Hash + Clone + 'static,
    V: BytesEncode<'a> + BytesDecode<'a> + Clone + 'static,
    C: BlockHistoryCache<'a, V> + BytesDecode<'a> + BytesEncode<'a> + Clone + 'static,
{
    pub fn new(
        env: &'a Env,
        name: &str,
        parent_wtxn: &mut RwTxn,
    ) -> Result<BlockCachedDatabase<'a, K, V, C>> {
        let db: Database<K, V> = {
            let old_db = env.open_database::<K, V>(Some(name))?;
            if old_db.is_some() {
                old_db.unwrap()
            } else {
                env.create_database_with_txn(Some(name), parent_wtxn)?
            }
        };
        let cache_db = {
            let cache_name = format!("{}_cache", &name);
            let old_db = env.open_database::<K, C>(Some(&cache_name))?;
            if old_db.is_some() {
                old_db.unwrap()
            } else {
                env.create_database_with_txn(Some(&cache_name), parent_wtxn)?
            }
        };
        let cache = Map::new();
        Ok(BlockCachedDatabase {
            env: env,
            rtxn: env.read_txn()?,
            db,
            cache_db,
            cache,
            _phantom: std::marker::PhantomData,
        })
    }

    pub fn latest(&'a mut self, key: &'a K) -> Option<V> {
        if self.cache.contains_key(key) {
            let cache = self.cache.get(key).unwrap();
            return cache.latest();
        }
        let result = self.db.get(&self.rtxn, as_eitem(key));
        if result.is_err() {
            None
        } else {
            let value = result.unwrap().unwrap();
            Some(from_ditem::<V>(&value).clone())
        }
    }

    pub fn set(&mut self, block_number: U256, key: K, value: V) -> Result<()> {
        if self.cache.contains_key(&key) {
            let cache = self.cache.get_mut(&key).unwrap();
            cache.set(block_number, value);
        } else {
            self.load_cache_if_needed(&key, Some(block_number))?;
            let cache = self.cache.get_mut(&key).unwrap();
            cache.set(block_number, value);
        }
        Ok(())
    }

    pub fn commit(&mut self, parent_wtxn: &mut RwTxn) -> Result<()> {
        // TODO: Implement commit
        Ok(())
    }

    pub fn rollback(&mut self, parent_wtxn: &mut RwTxn, block_number: U256) {
        // TODO: Implement rollback
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    fn load_cache_if_needed(&mut self, key: &K, block_number: Option<U256>) -> Result<()> {
        if self.cache.contains_key(key) {
            return Ok(());
        }
        // TODO: Remove this to load from database
        let mut block_cache = C::new();
        block_cache.set_empty(block_number.unwrap_or(U256::ZERO));
        self.cache.insert(key.clone(), block_cache);

        // TODO: Database interactions
        // let rtxn = self.env.read_txn()?;
        // let result = self.cache_db.get(&rtxn, as_eitem(key));
        // if result.is_err() {
        //     // load latest value from db and create an empty or full cache at block_number
        //     let value = self.db.get(&rtxn, as_eitem(key));
        //     if value.is_err() {
        //         let mut new_cache = C::new();
        //         new_cache.set_empty(block_number);
        //         self.cache.insert(key.clone(), new_cache);
        //     } else {
        //         let value = value.unwrap().unwrap();
        //         let mut new_cache = C::new();
        //         new_cache.set(block_number, from_ditem(&value));
        //         self.cache.insert(key.clone(), new_cache);
        //     }
        // } else {
        //     let cache = result.unwrap().unwrap();
        //     self.cache.insert(key.clone(), from_ditem(&cache));
        // }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use heed::Env;
    use revm::primitives::AccountInfo;
    use revm::primitives::Address;
    use revm::primitives::B256;
    use revm::primitives::U256;

    use crate::cache::{BlockCachedDatabase, BlockHistoryCache, BlockHistoryCacheData};
    use crate::test_utils::create_test_env;
    use crate::types::{AccountInfoED, AddressED};

    #[test]
    fn test_cache_only() {
        let env = create_test_env().unwrap();
        let mut wtxn = env.write_txn().unwrap();
        let mut db = BlockCachedDatabase::<
            AddressED,
            AccountInfoED,
            BlockHistoryCacheData<AccountInfoED>,
        >::new(&env, "test_db", &mut wtxn)
        .unwrap();

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
        let _ = db.set(U256::from(1), address_ed.clone(), account_info.clone());

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
        let env = create_test_env().unwrap();
        let mut wtxn = env.write_txn().unwrap();
        let mut db = BlockCachedDatabase::<
            AddressED,
            AccountInfoED,
            BlockHistoryCacheData<AccountInfoED>,
        >::new(&env, "test_db", &mut wtxn)
        .unwrap();

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
        let _ = db.set(U256::from(1), address_ed.clone(), account_info.clone());

        let account_info = db.latest(&address_ed).unwrap();
        assert_eq!(account_info.0.balance, U256::from(100));
        assert_eq!(account_info.0.nonce, 1);
        assert_eq!(account_info.0.code_hash, B256::from([1; 32]));

        db.commit(&mut wtxn).unwrap();

        // TODO: Make this pass
        // verify the database content
        let real_db = db.db;

        let rtxn = env.read_txn().unwrap();
        let account_info = real_db.get(&rtxn, &address_ed).unwrap().unwrap();
        assert_eq!(account_info.0.balance, U256::from(100));
        assert_eq!(account_info.0.nonce, 1);
        assert_eq!(account_info.0.code_hash, B256::from([1; 32]));

        // verify the cache content
        let real_cache_db = db.cache_db;
        let cache: BlockHistoryCacheData<AccountInfoED> =
            real_cache_db.get(&rtxn, &address_ed).unwrap().unwrap();
        assert_eq!(cache.latest().unwrap().0.balance, U256::from(100));
        assert_eq!(cache.latest().unwrap().0.nonce, 1);
        assert_eq!(cache.latest().unwrap().0.code_hash, B256::from([1; 32]));
    }
}
