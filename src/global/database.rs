#![cfg(feature = "server")]

use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

use rocksdb::{Options, DB};

use crate::db::types::{Decode, Encode};
use crate::global::{
    Brc20ProgConfig, BITCOIN_RPC_NETWORK_KEY, DB_VERSION, DB_VERSION_KEY, EVM_RECORD_TRACES_KEY,
    PROTOCOL_VERSION, PROTOCOL_VERSION_KEY,
};

pub struct ConfigDatabase {
    db: DB,
    cache: HashMap<String, String>,
}

impl ConfigDatabase {
    pub fn new(path: &Path, name: &str) -> Result<Self, Box<dyn Error>> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_max_open_files(256);
        let db = DB::open(&opts, &path.join(Path::new(name)))?;
        Ok(Self { db, cache: HashMap::new() })
    }

    pub fn get(&self, key: String) -> Result<Option<String>, Box<dyn Error>> {
        if let Some(value) = self.cache.get(&key) {
            return Ok(Some(value.clone()));
        }
        Ok(self
            .db
            .get(&key.encode_vec())?
            .map_or(None, |value| String::decode_vec(&value).ok()))
    }

    pub fn set(&mut self, key: String, value: String) -> Result<(), Box<dyn Error>> {
        self.db.put(&key.encode_vec(), &value.encode_vec())?;
        self.cache.insert(key, value);
        Ok(())
    }

    pub fn flush(&self) -> Result<(), Box<dyn Error>> {
        self.db.flush().map_err(|e| e.into())
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
    let fresh_run = !db_path.read_dir()?.next().is_some();

    let mut config_database = ConfigDatabase::new(&Path::new(&config.db_path), "config")?;
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
        config_database.flush()?;
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
        let mut db = ConfigDatabase::new(&temp.path(), "config").unwrap();
        db.set("key".to_string(), "value".to_string()).unwrap();
        let value = db.get("key".to_string()).unwrap();
        assert_eq!(value, Some("value".to_string()));
    }

    #[test]
    fn test_config_database_not_found() {
        let temp = TempDir::new().unwrap();
        let db = ConfigDatabase::new(&temp.path(), "config").unwrap();
        let value = db.get("key".to_string()).unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn test_config_database_update() {
        let temp = TempDir::new().unwrap();
        let mut db = ConfigDatabase::new(&temp.path(), "config").unwrap();
        db.set("key".to_string(), "value".to_string()).unwrap();
        db.set("key".to_string(), "value2".to_string()).unwrap();
        let value = db.get("key".to_string()).unwrap();
        assert_eq!(value, Some("value2".to_string()));
    }

    #[test]
    fn test_config_database_validate() {
        let temp = TempDir::new().unwrap();
        let mut db = ConfigDatabase::new(&temp.path(), "config").unwrap();
        db.set("key".to_string(), "value".to_string()).unwrap();
        let result = db.validate("key", "value");
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_database_validate_mismatch() {
        let temp = TempDir::new().unwrap();
        let mut db = ConfigDatabase::new(&temp.path(), "config").unwrap();
        db.set("key".to_string(), "value".to_string()).unwrap();
        let result = db.validate("key", "value2");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_database_validate_not_found() {
        let temp = TempDir::new().unwrap();
        let mut db = ConfigDatabase::new(&temp.path(), "config").unwrap();
        let result = db.validate("key", "value");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_database_validate_not_found_mismatch() {
        let temp = TempDir::new().unwrap();
        let mut db = ConfigDatabase::new(&temp.path(), "config").unwrap();
        db.set("key".to_string(), "value".to_string()).unwrap();
        let result = db.validate("key", "value2");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_database_flush() {
        let temp = TempDir::new().unwrap();
        let mut db = ConfigDatabase::new(&temp.path(), "config").unwrap();
        db.set("key".to_string(), "value".to_string()).unwrap();
        let result = db.flush();
        assert!(result.is_ok());
    }
}