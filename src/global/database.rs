#![cfg(feature = "server")]

use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;

use rocksdb::{WriteBatch, DB};

use crate::db::store::{self, column_family, MIGRATING_DIR};
use crate::db::types::{Decode, Encode};
use crate::global::{
    Brc20ProgConfig, BITCOIN_RPC_NETWORK_KEY, DB_VERSION, DB_VERSION_KEY, EVM_RECORD_TRACES_KEY,
    PROTOCOL_VERSION, PROTOCOL_VERSION_KEY,
};

/// String key-value database in a column family of the engine store.
///
/// Values set since the last commit are pending: reads see them, and the next commit writes them.
pub struct ConfigDatabase {
    db: Arc<DB>,
    name: String,
    cache: HashMap<String, String>,
    pending: HashMap<String, String>,
}

impl ConfigDatabase {
    pub fn new(db: Arc<DB>, name: &str) -> Result<Self, Box<dyn Error>> {
        column_family(&db, name)?;
        Ok(Self {
            db,
            name: name.to_string(),
            cache: HashMap::new(),
            pending: HashMap::new(),
        })
    }

    pub fn get(&self, key: String) -> Result<Option<String>, Box<dyn Error>> {
        if let Some(value) = self.cache.get(&key) {
            return Ok(Some(value.clone()));
        }
        Ok(self
            .db
            .get_cf(column_family(&self.db, &self.name)?, key.encode_vec())?
            .map_or(None, |value| String::decode_vec(&value).ok()))
    }

    pub fn set(&mut self, key: String, value: String) -> Result<(), Box<dyn Error>> {
        self.pending.insert(key.clone(), value.clone());
        self.cache.insert(key, value);
        Ok(())
    }

    /// Adds the pending values to a commit's write batch. They stay pending until
    /// [`ConfigDatabase::clear_pending`].
    pub fn stage_commit(&self, batch: &mut WriteBatch) -> Result<(), Box<dyn Error>> {
        let cf = column_family(&self.db, &self.name)?;
        for (key, value) in &self.pending {
            batch.put_cf(cf, key.encode_vec(), value.encode_vec());
        }
        Ok(())
    }

    /// Marks the pending values as written.
    pub fn clear_pending(&mut self) {
        self.pending.clear();
    }

    /// Writes the pending values in their own batch.
    pub fn commit(&mut self) -> Result<(), Box<dyn Error>> {
        let mut batch = WriteBatch::default();
        self.stage_commit(&mut batch)?;
        self.db.write(batch)?;
        self.clear_pending();
        Ok(())
    }

    pub fn validate(&mut self, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
        match self.get(key.to_string())? {
            Some(db_value) => {
                if db_value != value.to_string() {
                    return Err(format!(
                        "Config for {} mismatch: expected {}, found {}",
                        key, value, db_value
                    )
                    .into());
                }
            }
            None => {
                return Err(format!("Config for {} not found: expected {}", key, value).into());
            }
        };
        Ok(())
    }
}

pub fn validate_config_database(config: &Brc20ProgConfig) -> Result<(), Box<dyn Error>> {
    let db_path = Path::new(&config.db_path);
    if !db_path.exists() {
        std::fs::create_dir_all(db_path)?;
    } else {
        if !db_path.is_dir() {
            return Err(format!("{} is not a directory", config.db_path).into());
        }
    }
    // A store left half-built by an interrupted first start counts as no data.
    let fresh_run = db_path
        .read_dir()?
        .filter_map(|entry| entry.ok())
        .all(|entry| entry.file_name() == MIGRATING_DIR);

    let mut config_database = ConfigDatabase::new(store::open(db_path)?, "config")?;
    if fresh_run {
        config_database.set(DB_VERSION_KEY.clone(), DB_VERSION.to_string())?;
        config_database.set(PROTOCOL_VERSION_KEY.clone(), PROTOCOL_VERSION.to_string())?;
        config_database.set(
            BITCOIN_RPC_NETWORK_KEY.clone(),
            config.bitcoin_rpc_network.clone(),
        )?;
        config_database.set(
            EVM_RECORD_TRACES_KEY.clone(),
            config.evm_record_traces.to_string(),
        )?;
        config_database.commit()?;
    } else {
        config_database.validate(&*DB_VERSION_KEY, &DB_VERSION.to_string())?;
        config_database.validate(&*PROTOCOL_VERSION_KEY, &PROTOCOL_VERSION.to_string())?;
        config_database.validate(&*BITCOIN_RPC_NETWORK_KEY, &config.bitcoin_rpc_network)?;
        config_database.validate(
            &*EVM_RECORD_TRACES_KEY,
            &config.evm_record_traces.to_string(),
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_config_database() {
        let temp = TempDir::new().unwrap();
        let mut db = ConfigDatabase::new(store::open(temp.path()).unwrap(), "config").unwrap();
        db.set("key".to_string(), "value".to_string()).unwrap();
        let value = db.get("key".to_string()).unwrap();
        assert_eq!(value, Some("value".to_string()));
    }

    #[test]
    fn test_config_database_not_found() {
        let temp = TempDir::new().unwrap();
        let db = ConfigDatabase::new(store::open(temp.path()).unwrap(), "config").unwrap();
        let value = db.get("key".to_string()).unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn test_config_database_update() {
        let temp = TempDir::new().unwrap();
        let mut db = ConfigDatabase::new(store::open(temp.path()).unwrap(), "config").unwrap();
        db.set("key".to_string(), "value".to_string()).unwrap();
        db.set("key".to_string(), "value2".to_string()).unwrap();
        let value = db.get("key".to_string()).unwrap();
        assert_eq!(value, Some("value2".to_string()));
    }

    #[test]
    fn test_config_database_validate() {
        let temp = TempDir::new().unwrap();
        let mut db = ConfigDatabase::new(store::open(temp.path()).unwrap(), "config").unwrap();
        db.set("key".to_string(), "value".to_string()).unwrap();
        let result = db.validate("key", "value");
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_database_validate_mismatch() {
        let temp = TempDir::new().unwrap();
        let mut db = ConfigDatabase::new(store::open(temp.path()).unwrap(), "config").unwrap();
        db.set("key".to_string(), "value".to_string()).unwrap();
        let result = db.validate("key", "value2");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_database_validate_not_found() {
        let temp = TempDir::new().unwrap();
        let mut db = ConfigDatabase::new(store::open(temp.path()).unwrap(), "config").unwrap();
        let result = db.validate("key", "value");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_database_validate_not_found_mismatch() {
        let temp = TempDir::new().unwrap();
        let mut db = ConfigDatabase::new(store::open(temp.path()).unwrap(), "config").unwrap();
        db.set("key".to_string(), "value".to_string()).unwrap();
        let result = db.validate("key", "value2");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_database_commit() {
        let temp = TempDir::new().unwrap();
        {
            let mut db = ConfigDatabase::new(store::open(temp.path()).unwrap(), "config").unwrap();
            db.set("key".to_string(), "value".to_string()).unwrap();
            assert!(db.commit().is_ok());
        }
        let db = ConfigDatabase::new(store::open(temp.path()).unwrap(), "config").unwrap();
        assert_eq!(
            db.get("key".to_string()).unwrap(),
            Some("value".to_string())
        );
    }

    #[test]
    fn test_config_database_uncommitted_value_is_not_stored() {
        let temp = TempDir::new().unwrap();
        {
            let mut db = ConfigDatabase::new(store::open(temp.path()).unwrap(), "config").unwrap();
            db.set("key".to_string(), "value".to_string()).unwrap();
        }
        let db = ConfigDatabase::new(store::open(temp.path()).unwrap(), "config").unwrap();
        assert_eq!(db.get("key".to_string()).unwrap(), None);
    }
}
