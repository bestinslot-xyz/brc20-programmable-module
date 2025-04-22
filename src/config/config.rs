use std::error::Error;
use std::path::Path;

use crate::config::database::ConfigDatabase;

lazy_static::lazy_static! {
    static ref DB_VERSION: u32 = 1;
    static ref DB_PATH: String = std::env::var("BRC20_PROG_DB_PATH").unwrap_or("target/db".to_string());

    static ref BRC20_PROG_RPC_SERVER_ENABLE_AUTH: bool = std::env::var("BRC20_PROG_RPC_SERVER_ENABLE_AUTH").map(|x| x == "true").unwrap_or(false);
    static ref BRC20_PROG_RPC_SERVER_USER: Option<String> = std::env::var("BRC20_PROG_RPC_SERVER_USER").ok();
    static ref BRC20_PROG_RPC_SERVER_PASSWORD: Option<String> = std::env::var("BRC20_PROG_RPC_SERVER_PASSWORD").ok();
    static ref BRC20_PROG_RPC_SERVER_URL: String = std::env::var("BRC20_PROG_RPC_SERVER_URL").unwrap_or("127.0.0.1:18545".to_string());

    static ref BRC20_PROG_BALANCE_SERVER_URL: String = std::env::var("BRC20_PROG_BALANCE_SERVER_URL").unwrap_or("http://localhost:18546".to_string());

    static ref EVM_RECORD_TRACES: bool = std::env::var("EVM_RECORD_TRACES").map(|x| x == "true").unwrap_or(false);

    static ref BITCOIN_RPC_URL: String = std::env::var("BITCOIN_RPC_URL").unwrap_or("http://localhost:38332".to_string());
    static ref BITCOIN_RPC_USER: String = std::env::var("BITCOIN_RPC_USER").unwrap_or("user".to_string());
    static ref BITCOIN_RPC_PASSWORD: String = std::env::var("BITCOIN_RPC_PASSWORD").unwrap_or("password".to_string());
    static ref BITCOIN_RPC_NETWORK: String = std::env::var("BITCOIN_RPC_NETWORK").unwrap_or("signet".to_string());
    static ref CARGO_PKG_VERSION: String = {
        let version = env!("CARGO_PKG_VERSION");
        if version.is_empty() {
            "0.0.0".to_string()
        } else {
            version.to_string()
        }
    };

    pub static ref BRC20_PROG_CONFIG: Brc20ProgConfig = Brc20ProgConfig::new();
}

pub struct Brc20ProgConfig {
    pub brc20_prog_rpc_server_url: String,
    pub brc20_prog_rpc_server_enable_auth: bool,
    pub brc20_prog_rpc_server_user: Option<String>,
    pub brc20_prog_rpc_server_password: Option<String>,
    pub brc20_balance_server_url: String,
    pub evm_record_traces: bool,
    pub bitcoin_rpc_url: String,
    pub bitcoin_rpc_user: String,
    pub bitcoin_rpc_password: String,
    pub bitcoin_rpc_network: String,
    pub pkg_version: String,
    pub db_path: String,
}

impl Brc20ProgConfig {
    fn new() -> Self {
        Self {
            brc20_prog_rpc_server_url: BRC20_PROG_RPC_SERVER_URL.to_string(),
            brc20_prog_rpc_server_enable_auth: *BRC20_PROG_RPC_SERVER_ENABLE_AUTH,
            brc20_prog_rpc_server_user: BRC20_PROG_RPC_SERVER_USER.clone(),
            brc20_prog_rpc_server_password: BRC20_PROG_RPC_SERVER_PASSWORD.clone(),
            brc20_balance_server_url: BRC20_PROG_BALANCE_SERVER_URL.to_string(),
            evm_record_traces: *EVM_RECORD_TRACES,
            bitcoin_rpc_url: BITCOIN_RPC_URL.to_string(),
            bitcoin_rpc_user: BITCOIN_RPC_USER.to_string(),
            bitcoin_rpc_password: BITCOIN_RPC_PASSWORD.to_string(),
            bitcoin_rpc_network: BITCOIN_RPC_NETWORK.to_string(),
            pkg_version: CARGO_PKG_VERSION.to_string(),
            db_path: DB_PATH.to_string(),
        }
    }
}

pub fn validate_config_database() -> Result<(), Box<dyn Error>> {
    let fresh_run = !Path::new(&*DB_PATH).exists();
    let mut config_database = ConfigDatabase::new(&Path::new(&*DB_PATH), "config")?;
    if fresh_run {
        config_database.set("db_version".to_string(), DB_VERSION.to_string())?;
        config_database.set(
            "bitcoin_network".to_string(),
            BITCOIN_RPC_NETWORK.to_string(),
        )?;
        config_database.set(
            "evm_record_traces".to_string(),
            EVM_RECORD_TRACES.to_string(),
        )?;
    } else {
        config_database.validate("db_version", &DB_VERSION.to_string())?;
        config_database.validate("bitcoin_network", &BITCOIN_RPC_NETWORK.to_string())?;
        config_database.validate("evm_record_traces", &EVM_RECORD_TRACES.to_string())?;
    }
    Ok(())
}

pub fn validate_config() -> Result<(), Box<dyn Error>> {
    validate_config_database()?;

    if *BRC20_PROG_RPC_SERVER_ENABLE_AUTH
        && (BRC20_PROG_RPC_SERVER_USER.is_none() || BRC20_PROG_RPC_SERVER_PASSWORD.is_none())
    {
        return Err("Authentication is enabled but no username or password is set".into());
    }

    if BRC20_PROG_RPC_SERVER_URL.is_empty() {
        return Err("RPC server URL is empty".into());
    }

    if BRC20_PROG_BALANCE_SERVER_URL.is_empty() {
        return Err("BRC20 balance server URL is empty".into());
    }

    if !BRC20_PROG_BALANCE_SERVER_URL.starts_with("http://")
        && !BRC20_PROG_BALANCE_SERVER_URL.starts_with("https://")
    {
        return Err("BRC20 balance server URL must start with http:// or https://".into());
    }

    Ok(())
}
