// NOTE: Limbs are little-endian
// DB needs big-endian bytes

use std::error::Error;
use std::path::Path;
use std::borrow::Cow;
use std::fs;

use core::fmt;

use heed::{Database, EnvOpenOptions, BytesEncode, BytesDecode, Env, RwTxn};

use revm::primitives::ruint::aliases::{U256, U160};
use revm::primitives::ruint::Uint;
use revm::primitives::alloy_primitives::Bytes;
use revm::primitives::db::Database as DatabaseTrait;
use revm::primitives::{Address, AccountInfo, Account, Bytecode, B256, FixedBytes};

use hashbrown::HashMap as Map;

pub struct DB {
  pub env: Option<Env>,
  pub db_account_memory_map: Option<Database<U512ED, U256ED>>,
  pub db_code_map: Option<Database<B256ED, BytecodeED>>,
  pub db_account_map: Option<Database<AddressED, AccountInfoED>>,
  pub db_block_hash_map: Option<Database<U256ED, B256ED>>,
  pub db_block_timestamp_map: Option<Database<U256ED, U256ED>>,
  pub db_block_gas_used_map: Option<Database<U256ED, U256ED>>,
  pub db_block_mine_tm_map: Option<Database<U256ED, U256ED>>,
  pub code_cache: Map<B256, Bytecode>,
}

impl DB {
  pub fn new() -> Result<Self, Box<dyn Error>> {
    let path = Path::new("target").join("heed.mdb");
    fs::create_dir_all(&path)?;

    let env = EnvOpenOptions::new()
        .map_size(100 * 1024 * 1024 * 1024) // 100GB
        .max_dbs(3000)
        .open(path)?;
    
    let mut wtxn = env.write_txn()?;
    let db_account_memory_map: Database<U512ED, U256ED> = {
      let old_db = env.open_database::<U512ED, U256ED>(Some("account_memory_map"))?;
      if old_db.is_some() {
        old_db.unwrap()
      } else {
        env.create_database_with_txn(Some("account_memory_map"), &mut wtxn)?
      }
    };
    let db_code_map: Database<B256ED, BytecodeED> = {
      let old_db = env.open_database::<B256ED, BytecodeED>(Some("code_map"))?;
      if old_db.is_some() {
        old_db.unwrap()
      } else {
        env.create_database_with_txn(Some("code_map"), &mut wtxn)?
      }
    };
    let db_account_map: Database<AddressED, AccountInfoED> = {
      let old_db = env.open_database::<AddressED, AccountInfoED>(Some("account_map"))?;
      if old_db.is_some() {
        old_db.unwrap()
      } else {
        env.create_database_with_txn(Some("account_map"), &mut wtxn)?
      }
    };
    let db_block_hash_map: Database<U256ED, B256ED> = {
      let old_db = env.open_database::<U256ED, B256ED>(Some("block_hash"))?;
      if old_db.is_some() {
        old_db.unwrap()
      } else {
        env.create_database_with_txn(Some("block_hash"), &mut wtxn)?
      }
    };
    let db_block_timestamp_map: Database<U256ED, U256ED> = {
      let old_db = env.open_database::<U256ED, U256ED>(Some("block_ts"))?;
      if old_db.is_some() {
        old_db.unwrap()
      } else {
        env.create_database_with_txn(Some("block_ts"), &mut wtxn)?
      }
    };
    let db_block_gas_used_map: Database<U256ED, U256ED> = {
      let old_db = env.open_database::<U256ED, U256ED>(Some("gas_used"))?;
      if old_db.is_some() {
        old_db.unwrap()
      } else {
        env.create_database_with_txn(Some("gas_used"), &mut wtxn)?
      }
    };
    let db_block_mine_tm_map: Database<U256ED, U256ED> = {
      let old_db = env.open_database::<U256ED, U256ED>(Some("mine_tm"))?;
      if old_db.is_some() {
        old_db.unwrap()
      } else {
        env.create_database_with_txn(Some("mine_tm"), &mut wtxn)?
      }
    };
    wtxn.commit()?;

    Ok(Self {
      env: Some(env),
      db_account_memory_map: Some(db_account_memory_map),
      db_code_map: Some(db_code_map),
      db_account_map: Some(db_account_map),
      db_block_hash_map: Some(db_block_hash_map),
      db_block_timestamp_map: Some(db_block_timestamp_map),
      db_block_gas_used_map: Some(db_block_gas_used_map),
      db_block_mine_tm_map: Some(db_block_mine_tm_map),
      code_cache: Map::new(),
    })
  }

  pub fn get_write_txn(&self) -> Result<RwTxn, Box<dyn Error>> {
    Ok(self.env.as_ref().unwrap().write_txn()?)
  }

  pub fn read_from_account_memory_map(&self, account: Address, mem_loc: U256) -> Result<Option<U256>, Box<dyn Error>> {
    let rtxn = self.env.as_ref().unwrap().read_txn()?;

    let ret: Option<U256ED> = self.db_account_memory_map.unwrap().get(&rtxn, &U512ED::from_addr_u256(account, mem_loc))?;
    Ok(ret.map(|x| x.0))
  }

  pub fn set_account_memory_map_with_txn(&self, account: Address, mem_loc: U256, value: U256, parent_wtxn: &mut RwTxn) -> Result<(), Box<dyn Error>> {
    self.db_account_memory_map.unwrap().put(parent_wtxn, &U512ED::from_addr_u256(account, mem_loc), &U256ED::from_u256(value))?;
    Ok(())
  }

  pub fn remove_from_account_memory_map_with_txn(&self, account: Address, parent_wtxn: &mut RwTxn) -> Result<(), Box<dyn Error>> {
    let min_loc = U512ED::from_addr_u256(account, U256::ZERO);
    let max_loc = U512ED::from_addr_u256(account, U256::MAX);
    let range = min_loc..=max_loc;
    self.db_account_memory_map.unwrap().delete_range(parent_wtxn, &range)?;
    Ok(())
  }

  pub fn read_from_code_map(&mut self, code_hash: B256) -> Result<Option<Bytecode>, Box<dyn Error>> {
    if self.code_cache.contains_key(&code_hash) {
      return Ok(Some(self.code_cache.get(&code_hash).unwrap().clone()));
    }

    let rtxn = self.env.as_ref().unwrap().read_txn()?;

    let ret: Option<BytecodeED> = self.db_code_map.unwrap().get(&rtxn, &B256ED::from_b256(code_hash))?;

    if ret.is_some() {
      let v = ret.unwrap().0;
      self.code_cache.insert(code_hash, v.clone());
      return Ok(Some(v));
    }

    Ok(ret.map(|x| x.0))
  }

  pub fn set_code_map_with_txn(&self, code_hash: B256, bytecode: Bytecode, parent_wtxn: &mut RwTxn) -> Result<(), Box<dyn Error>> {
    self.db_code_map.unwrap().put(parent_wtxn, &B256ED::from_b256(code_hash), &BytecodeED::from_bytecode(bytecode))?;
    Ok(())
  }

  pub fn read_from_account_map(&self, account: Address) -> Result<Option<AccountInfo>, Box<dyn Error>> {
    let rtxn = self.env.as_ref().unwrap().read_txn()?;

    let ret: Option<AccountInfoED> = self.db_account_map.unwrap().get(&rtxn, &AddressED::from_addr(account))?;
    Ok(ret.map(|x| x.0))
  }

  pub fn set_account_map_with_txn(&self, account: Address, value: AccountInfo, parent_wtxn: &mut RwTxn) -> Result<(), Box<dyn Error>> {
    self.db_account_map.unwrap().put(parent_wtxn, &AddressED::from_addr(account), &AccountInfoED::from_account_info(value))?;
    Ok(())
  }

  pub fn remove_from_account_map_with_txn(&self, account: Address, parent_wtxn: &mut RwTxn) -> Result<(), Box<dyn Error>> {
    self.db_account_map.unwrap().delete(parent_wtxn, &AddressED::from_addr(account))?;
    Ok(())
  }

  pub fn read_from_block_hashes(&self, number: U256) -> Result<Option<B256>, Box<dyn Error>> {
    let rtxn = self.env.as_ref().unwrap().read_txn()?;

    let ret: Option<B256ED> = self.db_block_hash_map.unwrap().get(&rtxn, &U256ED::from_u256(number))?;
    Ok(ret.map(|x| x.0))
  }

  pub fn set_block_hash_with_txn(&self, number: U256, value: B256, parent_wtxn: &mut RwTxn) -> Result<(), Box<dyn Error>> {
    self.db_block_hash_map.unwrap().put(parent_wtxn, &U256ED::from_u256(number), &B256ED::from_b256(value))?;
    Ok(())
  }

  pub fn read_from_block_timestamps(&self, number: U256) -> Result<Option<U256>, Box<dyn Error>> {
    let rtxn = self.env.as_ref().unwrap().read_txn()?;

    let ret: Option<U256ED> = self.db_block_timestamp_map.unwrap().get(&rtxn, &U256ED::from_u256(number))?;
    Ok(ret.map(|x| x.0))
  }

  pub fn set_block_timestamps_with_txn(&self, number: U256, value: U256, parent_wtxn: &mut RwTxn) -> Result<(), Box<dyn Error>> {
    self.db_block_timestamp_map.unwrap().put(parent_wtxn, &U256ED::from_u256(number), &U256ED::from_u256(value))?;
    Ok(())
  }

  pub fn read_from_block_gas_used(&self, number: U256) -> Result<Option<U256>, Box<dyn Error>> {
    let rtxn = self.env.as_ref().unwrap().read_txn()?;

    let ret: Option<U256ED> = self.db_block_gas_used_map.unwrap().get(&rtxn, &U256ED::from_u256(number))?;
    Ok(ret.map(|x| x.0))
  }

  pub fn set_block_gas_used_with_txn(&self, number: U256, value: U256, parent_wtxn: &mut RwTxn) -> Result<(), Box<dyn Error>> {
    self.db_block_gas_used_map.unwrap().put(parent_wtxn, &U256ED::from_u256(number), &U256ED::from_u256(value))?;
    Ok(())
  }

  pub fn read_from_block_mine_tm(&self, number: U256) -> Result<Option<U256>, Box<dyn Error>> {
    let rtxn = self.env.as_ref().unwrap().read_txn()?;

    let ret: Option<U256ED> = self.db_block_mine_tm_map.unwrap().get(&rtxn, &U256ED::from_u256(number))?;
    Ok(ret.map(|x| x.0))
  }

  pub fn set_block_mine_tm_with_txn(&self, number: U256, value: U256, parent_wtxn: &mut RwTxn) -> Result<(), Box<dyn Error>> {
    self.db_block_mine_tm_map.unwrap().put(parent_wtxn, &U256ED::from_u256(number), &U256ED::from_u256(value))?;
    Ok(())
  }

  pub fn get_latest_block_hash(&self) -> Result<Option<(U256, B256)>, Box<dyn Error>> {
    let rtxn = self.env.as_ref().unwrap().read_txn()?;

    let ret: Option<(U256ED, B256ED)> = self.db_block_hash_map.unwrap().last(&rtxn)?;
    Ok(ret.map(|x| (x.0.0, x.1.0)))
  }

  pub fn commit(&mut self, changes: Map<Address, Account>, txhash: &String, block_number: &U256) { // TODO: save changes for reorg!!
    // println!("commit {:?}", changes);
    let mut wtxn = self.get_write_txn().unwrap();
    for (address, account) in changes {
      if !account.is_touched() { continue; }
      if account.is_selfdestructed() { // TODO: check if working correctly!!
        self.remove_from_account_map_with_txn(address, &mut wtxn).unwrap();
        self.remove_from_account_memory_map_with_txn(address, &mut wtxn).unwrap();
        continue;
      }
      let mut acc_info = AccountInfo::default();
      acc_info.balance = account.info.balance;
      acc_info.nonce = account.info.nonce;
      acc_info.code_hash = account.info.code_hash;
      self.set_account_map_with_txn(address, acc_info, &mut wtxn).unwrap();

      let is_newly_created = account.is_created();
      if is_newly_created { // TODO: can contract change other than creation??
        self.set_code_map_with_txn(account.info.code_hash, account.info.code.unwrap(), &mut wtxn).unwrap();
      }

      for (loc, slot) in account.storage {
        if !slot.is_changed() { continue; }
        self.set_account_memory_map_with_txn(address, loc, slot.present_value(), &mut wtxn).unwrap();
      }
    }
    wtxn.commit().unwrap();
  }
}

impl Default for DB {
  fn default() -> Self {
    Self {
      env: None,
      db_account_memory_map: None,
      db_code_map: None,
      db_account_map: None,
      db_block_hash_map: None,
      db_block_timestamp_map: None,
      db_block_gas_used_map: None,
      db_block_mine_tm_map: None,
      code_cache: Map::new(),
    }
  }
}

impl DatabaseTrait for DB {
  type Error = Box<dyn Error>;

  /// Get basic account information.
  fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
    // println!("basic {}", address);
    let res = self.read_from_account_map(address)?;
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
    self.read_from_code_map(code_hash).map(|x| x.unwrap_or(Bytecode::default()))
  }

  /// Get storage value of address at index.
  fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
    // println!("storage {} {}", address, index);
    self.read_from_account_memory_map(address, index).map(|x| x.unwrap_or(U256::ZERO))
  }

  /// Get block hash by block number.
  fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
    // println!("block_hash {}", number);
    self.read_from_block_hashes(number).map(|x| x.unwrap_or(B256::ZERO))
  }
}


pub struct UintEncodeDecode<const BITS: usize, const LIMBS: usize>(Uint<BITS, LIMBS>);
type U512ED = UintEncodeDecode<512, 8>;
type U256ED = UintEncodeDecode<256, 4>;

impl U512ED {
  pub fn from_addr_u256(a: Address, b: U256) -> Self {
    let addr_u160 = U160::from_be_bytes(a.0 .0);
    let limbs1 = addr_u160.as_limbs();
    let limbs2 = b.as_limbs();
    let mut limbs = [0u64; 8];
    for i in 0..4 {
      limbs[i + 0] = limbs2[i];
    }
    for i in 0..3 {
      limbs[i + 4] = u64::from(limbs1[i]);
    }

    Self(Uint::from_limbs(limbs))
  }
}
impl U256ED {
  pub fn from_u256(a: U256) -> Self {
    Self(a)
  }
}

impl<'a, const BITS: usize, const LIMBS: usize> BytesEncode<'a> for UintEncodeDecode<BITS, LIMBS> {
  type EItem = UintEncodeDecode<BITS, LIMBS>;

  fn bytes_encode(item: &'a Self::EItem) -> Result<Cow<'a, [u8]>, Box<dyn Error>>{
    let mut limbs = item.0.as_limbs().to_vec();
    limbs.reverse();
    let bytes = limbs.iter().flat_map(|limb| limb.to_be_bytes().to_vec()).collect::<Vec<u8>>();
    Ok(Cow::Owned(bytes))
  }
}
impl<'a, const BITS: usize, const LIMBS: usize> BytesDecode<'a> for UintEncodeDecode<BITS, LIMBS> {
  type DItem = UintEncodeDecode<BITS, LIMBS>;

  fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem, Box<dyn Error>>{
    let mut limbs = [0u64; LIMBS];
    for (i, limb) in limbs.iter_mut().enumerate() {
      let start = (LIMBS - 1 - i) * 8;
      let end = start + 8;
      let bytes = &bytes[start..end];
      *limb = u64::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]);
    }
    Ok(UintEncodeDecode(Uint::from_limbs(limbs)))
  }
}
impl<const BITS: usize, const LIMBS: usize> fmt::Display for UintEncodeDecode<BITS, LIMBS> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

pub struct BEncodeDecode<const N: usize>(FixedBytes<N>);
type B256ED = BEncodeDecode<32>;

impl B256ED {
  pub fn from_b256(a: B256) -> Self {
    Self(a)
  }
}

impl<'a, const N: usize> BytesEncode<'a> for BEncodeDecode<N> {
  type EItem = BEncodeDecode<N>;

  fn bytes_encode(item: &'a Self::EItem) -> Result<Cow<'a, [u8]>, Box<dyn Error>>{
    let bytes = item.0.as_slice().to_vec();
    Ok(Cow::Owned(bytes))
  }
}
impl<'a, const N: usize> BytesDecode<'a> for BEncodeDecode<N> {
  type DItem = BEncodeDecode<N>;

  fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem, Box<dyn Error>>{
    let mut arr = [0u8; N];
    arr.copy_from_slice(&bytes);
    Ok(BEncodeDecode(FixedBytes::from(arr)))
  }
}

pub struct BytecodeED(Bytecode);

impl BytecodeED {
  pub fn from_bytecode(a: Bytecode) -> Self {
    Self(a)
  }
}

impl<'a> BytesEncode<'a> for BytecodeED {
  type EItem = BytecodeED;

  fn bytes_encode(item: &'a Self::EItem) -> Result<Cow<'a, [u8]>, Box<dyn Error>>{
    let bytes = item.0.bytecode.0.to_vec();
    Ok(Cow::Owned(bytes))
  }
}
impl<'a> BytesDecode<'a> for BytecodeED {
  type DItem = BytecodeED;

  fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem, Box<dyn Error>>{
    Ok(BytecodeED(Bytecode::new_raw(Bytes::from(bytes.to_vec()))))
  }
}

pub struct AddressED(Address);

impl AddressED {
  pub fn from_addr(a: Address) -> Self {
    Self(a)
  }
}

impl<'a> BytesEncode<'a> for AddressED {
  type EItem = AddressED;

  fn bytes_encode(item: &'a Self::EItem) -> Result<Cow<'a, [u8]>, Box<dyn Error>>{
    let bytes = item.0.0.to_vec();
    Ok(Cow::Owned(bytes))
  }
}
impl<'a> BytesDecode<'a> for AddressED {
  type DItem = AddressED;

  fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem, Box<dyn Error>>{
    let mut limbs = [0u64; 3];
    for (i, limb) in limbs.iter_mut().enumerate() {
      let start = i * 8;
      let end = start + 8;
      let bytes = &bytes[start..end];
      *limb = u64::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]);
    }
    Ok(AddressED(Address::from(U160::from_limbs(limbs))))
  }
}

pub struct AccountInfoED(AccountInfo);

impl AccountInfoED {
  pub fn from_account_info(a: AccountInfo) -> Self {
    Self(a)
  }
}

impl<'a> BytesEncode<'a> for AccountInfoED {
  type EItem = AccountInfoED;

  fn bytes_encode(item: &'a Self::EItem) -> Result<Cow<'a, [u8]>, Box<dyn Error>>{
    let mut bytes = Vec::new();
    for limb in item.0.balance.as_limbs().iter() {
      bytes.extend_from_slice(&limb.to_be_bytes());
    }
    bytes.extend_from_slice(&item.0.nonce.to_be_bytes());
    bytes.extend_from_slice(&item.0.code_hash.0.to_vec());
    Ok(Cow::Owned(bytes))
  }
}
impl<'a> BytesDecode<'a> for AccountInfoED {
  type DItem = AccountInfoED;

  fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem, Box<dyn Error>>{
    let mut limbs = [0u64; 4];
    for (i, limb) in limbs.iter_mut().enumerate() {
      let start = i * 8;
      let end = start + 8;
      let bytes = &bytes[start..end];
      *limb = u64::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]);
    }
    let balance = U256::from_limbs(limbs);
    let nonce = u64::from_be_bytes([bytes[32], bytes[33], bytes[34], bytes[35], bytes[36], bytes[37], bytes[38], bytes[39]]);
    let mut limbs = [0u64; 4];
    for (i, limb) in limbs.iter_mut().enumerate() {
      let start = (8 - i) * 8;
      let end = start + 8;
      let bytes = &bytes[start..end];
      *limb = u64::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]);
    }
    let code_hash_u = U256::from_limbs(limbs);
    let code_hash = B256::from(code_hash_u);
    Ok(AccountInfoED(AccountInfo {
      balance, nonce, code_hash, code: None
    }))
  }
}
