#![cfg(feature = "server")]

use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;

use rocksdb::{WriteBatch, WriteOptions, DB};

use crate::db::store::{self, column_family, LOCK_FILE, MIGRATING_DIR};
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

    /// Writes the pending values in their own batch, synced to disk.
    pub fn commit(&mut self) -> Result<(), Box<dyn Error>> {
        let mut batch = WriteBatch::default();
        self.stage_commit(&mut batch)?;
        let mut write_opts = WriteOptions::default();
        write_opts.set_sync(true);
        self.db.write_opt(batch, &write_opts)?;
        self.clear_pending();
        Ok(())
    }

    #[cfg(test)]
    pub fn validate(&mut self, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
        check_value(key, value, self.get(key.to_string())?)
    }
}

fn check_value(key: &str, value: &str, found: Option<String>) -> Result<(), Box<dyn Error>> {
    match found {
        Some(db_value) => {
            if db_value != value {
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

/// Checks the stored data dir configuration, read through `get`, against `config`.
fn check_settings(
    config: &Brc20ProgConfig,
    get: impl Fn(&str) -> Result<Option<String>, Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    let checks = [
        (DB_VERSION_KEY.as_str(), DB_VERSION.to_string()),
        (PROTOCOL_VERSION_KEY.as_str(), PROTOCOL_VERSION.to_string()),
        (
            BITCOIN_RPC_NETWORK_KEY.as_str(),
            config.bitcoin_rpc_network.clone(),
        ),
        (
            EVM_RECORD_TRACES_KEY.as_str(),
            config.evm_record_traces.to_string(),
        ),
    ];
    for (key, value) in checks {
        check_value(key, &value, get(key)?)?;
    }
    Ok(())
}

/// Checks the configuration of a data dir in the per-database layout before it migrates, reading
/// its `config` database without modifying it. A data dir without one fails like an empty
/// configuration.
fn check_per_database_settings(
    config: &Brc20ProgConfig,
    db_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let config_path = db_path.join("config");
    if !config_path.is_dir() {
        return check_settings(config, |_| Ok(None));
    }
    let old_config = store::open_per_database_dir(&config_path)?;
    check_settings(config, |key| {
        Ok(old_config
            .get(key.to_string().encode_vec())?
            .and_then(|value| String::decode_vec(&value).ok()))
    })
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
    // A store left half-built by an interrupted first start, and the open lock, count as no data.
    let fresh_run = db_path
        .read_dir()?
        .filter_map(|entry| entry.ok())
        .all(|entry| entry.file_name() == MIGRATING_DIR || entry.file_name() == LOCK_FILE);

    // A data dir about to migrate is checked first, so a mismatch leaves it untouched.
    if !fresh_run && store::needs_migration(db_path) {
        check_per_database_settings(config, db_path)?;
    }

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
        check_settings(config, |key| config_database.get(key.to_string()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::db::store::test_hooks::{copy_dir, file_tree, fixture_path};

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

    fn fixture_config(path: &Path, network: &str) -> Brc20ProgConfig {
        // The v0.16.6 fixture was written for mainnet with traces recorded.
        Brc20ProgConfig {
            db_path: path.to_str().unwrap().to_string(),
            bitcoin_rpc_network: network.to_string(),
            evm_record_traces: true,
            ..Default::default()
        }
    }

    #[test]
    fn test_mismatched_config_fails_before_migrating() {
        let temp = TempDir::new().unwrap();
        copy_dir(&fixture_path(), temp.path());

        let error = validate_config_database(&fixture_config(temp.path(), "signet"))
            .err()
            .unwrap()
            .to_string();
        assert_eq!(
            error,
            "Config for BITCOIN_RPC_NETWORK mismatch: expected signet, found bitcoin"
        );
        assert_eq!(file_tree(temp.path()), file_tree(&fixture_path()));
        assert!(!temp.path().join(store::ENGINE_DIR).exists());
        assert!(!temp.path().join(MIGRATING_DIR).exists());

        validate_config_database(&fixture_config(temp.path(), "bitcoin")).unwrap();
        assert!(temp.path().join(store::ENGINE_DIR).exists());
        assert!(!temp.path().join("config").exists());
    }

    #[test]
    fn test_per_database_dir_without_config_fails_before_migrating() {
        let temp = TempDir::new().unwrap();
        copy_dir(&fixture_path(), temp.path());
        std::fs::remove_dir_all(temp.path().join("config")).unwrap();
        let before = file_tree(temp.path());

        let error = validate_config_database(&fixture_config(temp.path(), "bitcoin"))
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("not found"), "{}", error);
        assert_eq!(file_tree(temp.path()), before);
        assert!(!temp.path().join(store::ENGINE_DIR).exists());
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
