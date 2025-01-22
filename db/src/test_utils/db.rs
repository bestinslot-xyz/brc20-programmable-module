use heed::{Env, EnvOpenOptions, Error};
use std::fs;
use std::path::Path;

pub fn create_test_env() -> Result<Env, Error> {
    let path = Path::new("test_db");
    if path.exists() {
        fs::remove_dir_all(path).unwrap();
    }
    fs::create_dir(path).unwrap();
    EnvOpenOptions::new()
        .map_size(20 * 1024 * 1024 * 1024) // 20GB // TODO: set this reasonably!!
        .max_dbs(3000)
        .open(path)
}
