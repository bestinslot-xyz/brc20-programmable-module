use heed::{Env, EnvOpenOptions};
use std::fs;
use std::path::Path;

pub struct TestEnvWrapper {
    pub env: Env,
}

impl Drop for TestEnvWrapper {
    fn drop(&mut self) {
        fs::remove_dir_all(self.env.path()).unwrap();
    }
}

pub fn create_test_env() -> TestEnvWrapper {
    let id = rand::random::<u64>().to_string();
    let path_str = format!("test_db{}", id);
    let path = Path::new(&path_str);
    fs::create_dir(path).unwrap();
    unsafe {
        TestEnvWrapper {
            env: EnvOpenOptions::new()
                .map_size(20 * 1024 * 1024 * 1024) // 20GB // TODO: set this reasonably!!
                .max_dbs(3000)
                .open(path)
                .unwrap(),
        }
    }
}
