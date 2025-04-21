use std::error::Error;
use std::path::Path;

use rocksdb::{Options, DB};

use crate::db::types::{Decode, Encode};

pub struct ConfigDatabase {
    db: DB,
}

impl ConfigDatabase {
    pub fn new(path: &Path, name: &str) -> Result<Self, Box<dyn Error>> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_max_open_files(256);
        let db = DB::open(&opts, &path.join(Path::new(name)))?;
        Ok(Self { db })
    }

    pub fn get(&self, key: String) -> Result<Option<String>, Box<dyn Error>> {
        Ok(self
            .db
            .get(&key.encode_vec())?
            .map_or(None, |value| String::decode_vec(&value).ok()))
    }

    pub fn set(&mut self, key: String, value: String) -> Result<(), Box<dyn Error>> {
        self.db.put(&key.encode_vec(), &value.encode_vec()).unwrap();
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
                return Err(format!(
                    "Config for {} not found: expected {}",
                    key, value
                )
                .into());
            }
        };
        Ok(())
    }
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
}
