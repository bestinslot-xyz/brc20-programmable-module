use std::env;

use alloy::primitives::Address;

use crate::global::shared_data::SharedData;

lazy_static::lazy_static! {
    pub(crate) static ref DB_VERSION_KEY: String = "DB_VERSION".to_string();
    pub(crate) static ref DB_VERSION: u32 = 5;

    pub(crate) static ref PROTOCOL_VERSION_KEY: String = "PROTOCOL_VERSION".to_string();
    pub(crate) static ref PROTOCOL_VERSION: u32 = 1;

    static ref DB_PATH_KEY: String = "BRC20_PROG_DB_PATH".to_string();
    static ref DB_PATH_DEFAULT: String = "target/db".to_string();

    static ref BRC20_PROG_RPC_SERVER_ENABLE_AUTH_KEY: String = "BRC20_PROG_RPC_SERVER_ENABLE_AUTH".to_string();
    static ref BRC20_PROG_RPC_SERVER_ENABLE_AUTH_DEFAULT: bool = false;

    static ref BRC20_PROG_RPC_SERVER_USER_KEY: String = "BRC20_PROG_RPC_SERVER_USER".to_string();
    static ref BRC20_PROG_RPC_SERVER_PASSWORD_KEY: String = "BRC20_PROG_RPC_SERVER_PASSWORD".to_string();

    static ref BRC20_PROG_RPC_SERVER_URL_KEY: String = "BRC20_PROG_RPC_SERVER_URL".to_string();
    static ref BRC20_PROG_RPC_SERVER_URL_DEFAULT: String = "127.0.0.1:18545".to_string();

    static ref BRC20_PROG_BALANCE_SERVER_URL_KEY: String = "BRC20_PROG_BALANCE_SERVER_URL".to_string();
    static ref BRC20_PROG_BALANCE_SERVER_URL_DEFAULT: String = "http://localhost:18546".to_string();

    static ref FAIL_ON_BRC20_BALANCE_SERVER_ERROR_KEY: String = "FAIL_ON_BRC20_BALANCE_SERVER_ERROR".to_string();
    static ref FAIL_ON_BRC20_BALANCE_SERVER_ERROR_DEFAULT: bool = true;

    static ref FAIL_ON_BITCOIN_RPC_ERROR_KEY: String = "FAIL_ON_BITCOIN_RPC_ERROR".to_string();
    static ref FAIL_ON_BITCOIN_RPC_ERROR_DEFAULT: bool = true;

    pub(crate) static ref EVM_RECORD_TRACES_KEY: String = "EVM_RECORD_TRACES".to_string();
    static ref EVM_RECORD_TRACES_DEFAULT: bool = false;

    static ref EVM_CALL_GAS_LIMIT_KEY: String = "EVM_CALL_GAS_LIMIT".to_string();
    static ref EVM_CALL_GAS_LIMIT: u64 = 1_000_000_000;

    static ref BITCOIN_RPC_URL_KEY: String = "BITCOIN_RPC_URL".to_string();
    static ref BITCOIN_RPC_URL_DEFAULT_SIGNET: String = "http://localhost:38332".to_string();

    static ref BITCOIN_RPC_USER_KEY: String = "BITCOIN_RPC_USER".to_string();
    static ref BITCOIN_RPC_PASSWORD_KEY: String = "BITCOIN_RPC_PASSWORD".to_string();

    pub(crate) static ref BITCOIN_RPC_NETWORK_KEY: String = "BITCOIN_RPC_NETWORK".to_string();
    static ref BITCOIN_RPC_NETWORK_DEFAULT_SIGNET: String = "signet".to_string();

    pub static ref CARGO_PKG_VERSION: String = {
        let version = env!("CARGO_PKG_VERSION");
        if version.is_empty() {
            "0.0.0".to_string()
        } else {
            version.to_string()
        }
    };

    pub static ref INDEXER_ADDRESS: Address = "0x0000000000000000000000000000000000003Ca6".parse().expect("Failed to parse indexer address");
    pub static ref INDEXER_ADDRESS_STRING: String = INDEXER_ADDRESS.to_string();
    pub static ref INVALID_ADDRESS: Address = "0x000000000000000000000000000000000000dead".parse().expect("Failed to parse invalid address");
}

lazy_static::lazy_static! {
    pub static ref CONFIG: SharedData<Brc20ProgConfig> = SharedData::new(Brc20ProgConfig::from_env());
}

pub const MAX_REORG_HISTORY_SIZE: u64 = 10; // 10 blocks, this is the maximum reorg history size
pub const MAX_BLOCK_SIZE: u64 = 4 * 1024 * 1024; // 4MB
pub const GAS_PER_BYTE: u64 = 12000; // 12K gas per byte

pub const MAX_FUTURE_TRANSACTION_NONCES: u64 = 10; // Maximum future transaction nonces allowed
pub const MAX_FUTURE_TRANSACTION_BLOCKS: u64 = 10; // Maximum future transaction block depth allowed

pub const GAS_PER_BITCOIN_RPC_CALL: u64 = 400000; // 400K gas per Bitcoin RPC call
pub const GAS_PER_BRC20_BALANCE_CALL: u64 = 200000; // 200K gas per BRC20 balance call
pub const GAS_PER_BIP_322_VERIFY: u64 = 20000; // 20K gas per BIP-322 verify call
pub const GAS_PER_LOCKED_PKSCRIPT: u64 = 20000; // 20K gas per locked pkscript call

const CHAIN_ID: u64 = 0x4252433230; // Mainnet Chain ID: BRC20 in hex
const CHAIN_ID_TESTNETS: u64 = 0x425243323073; // Testnets Chain ID: BRC20s in hex

pub const CALLDATA_LIMIT: usize = 1024 * 1024; // 1MB

/// Configuration for the BRC20 Prog server
/// This struct holds the configuration values for the BRC20 Prog server.
#[derive(Clone, Debug)]
pub struct Brc20ProgConfig {
    /// The URL of the BRC20 Prog RPC server
    pub brc20_prog_rpc_server_url: String,
    /// Whether to enable authentication for the BRC20 Prog RPC server
    pub brc20_prog_rpc_server_enable_auth: bool,
    /// The username for the BRC20 Prog RPC server, if authentication is enabled
    pub brc20_prog_rpc_server_user: Option<String>,
    /// The password for the BRC20 Prog RPC server, if authentication is enabled
    pub brc20_prog_rpc_server_password: Option<String>,
    /// The URL of the BRC20 balance server
    pub brc20_balance_server_url: String,
    /// Whether to record EVM traces
    pub evm_record_traces: bool,
    /// Gas limit for EVM calls, through eth_call or eth_estimate_gas
    pub evm_call_gas_limit: u64,
    /// The URL of the Bitcoin RPC server
    pub bitcoin_rpc_url: String,
    /// The username for the Bitcoin RPC server
    /// This is used for authentication with the Bitcoin RPC server
    pub bitcoin_rpc_user: String,
    /// The password for the Bitcoin RPC server
    /// This is used for authentication with the Bitcoin RPC server
    pub bitcoin_rpc_password: String,
    /// The network type for the Bitcoin RPC server
    /// This is used to determine the network (mainnet, testnet, signet, etc.) for the Bitcoin RPC server
    pub bitcoin_rpc_network: String,
    /// Chain ID for the Bitcoin RPC server
    /// This is used to determine the chain ID for the Bitcoin RPC server
    pub chain_id: u64,
    /// Whether to fail on BRC20 balance server errors, if set to true, the server will stop if BRC20 balance server is not reachable when needed
    pub fail_on_brc20_balance_server_error: bool,
    /// Whether to fail on Bitcoin RPC errors, if set to true, the server will stop if Bitcoin RPC server is not reachable when needed
    pub fail_on_bitcoin_rpc_error: bool,
    /// Database path
    pub db_path: String,
}

impl Default for Brc20ProgConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl Brc20ProgConfig {
    /// Creates a new instance of `Brc20ProgConfig` with the given parameters.
    ///
    /// Every argument corresponds to an environment variable.
    ///
    /// # Arguments
    /// * `brc20_prog_rpc_server_url` - The URL of the BRC20 Prog RPC server
    /// * `brc20_prog_rpc_server_enable_auth` - Whether to enable authentication for the BRC20 Prog RPC server
    /// * `brc20_prog_rpc_server_user` - The username for the BRC20 Prog RPC server, if authentication is enabled
    /// * `brc20_prog_rpc_server_password` - The password for the BRC20 Prog RPC server, if authentication is enabled
    /// * `brc20_balance_server_url` - The URL of the BRC20 balance server
    /// * `evm_call_gas_limit` - Gas limit for EVM calls (default: 1_000_000_000)
    /// * `evm_record_traces` - Whether to record EVM traces
    /// * `bitcoin_rpc_url` - The URL of the Bitcoin RPC server
    /// * `bitcoin_rpc_user` - The username for the Bitcoin RPC server
    /// * `bitcoin_rpc_password` - The password for the Bitcoin RPC server
    /// * `bitcoin_rpc_network` - The network type for the Bitcoin RPC server
    /// * `fail_on_bitcoin_rpc_error` - Whether to fail on Bitcoin RPC errors
    /// * `fail_on_brc20_balance_server_error` - Whether to fail on BRC20 balance server errors
    /// * `db_path` - The path to the database folder
    pub fn new(
        brc20_prog_rpc_server_url: String,
        brc20_prog_rpc_server_enable_auth: bool,
        brc20_prog_rpc_server_user: Option<String>,
        brc20_prog_rpc_server_password: Option<String>,
        brc20_balance_server_url: String,
        evm_record_traces: bool,
        evm_call_gas_limit: u64,
        bitcoin_rpc_url: String,
        bitcoin_rpc_user: String,
        bitcoin_rpc_password: String,
        bitcoin_rpc_network: String,
        chain_id: u64,
        fail_on_brc20_balance_server_error: bool,
        fail_on_bitcoin_rpc_error: bool,
        db_path: String,
    ) -> Self {
        Self {
            brc20_prog_rpc_server_url,
            brc20_prog_rpc_server_enable_auth,
            brc20_prog_rpc_server_user,
            brc20_prog_rpc_server_password,
            brc20_balance_server_url,
            evm_record_traces,
            evm_call_gas_limit,
            bitcoin_rpc_url,
            bitcoin_rpc_user,
            bitcoin_rpc_password,
            bitcoin_rpc_network,
            chain_id,
            fail_on_brc20_balance_server_error,
            fail_on_bitcoin_rpc_error,
            db_path,
        }
    }

    /// Creates a new instance of `Brc20ProgConfig` from environment variables
    ///
    /// This function reads the configuration values from environment variables and returns a new instance of `Brc20ProgConfig`.
    ///
    /// List of environment variables read:
    /// * `BRC20_PROG_DB_PATH` - The path to the database folder (Default: "target/db")
    /// * `BRC20_PROG_RPC_SERVER_URL` - The URL of the BRC20 Prog RPC server (Default: "127.0.0.1:18545")
    /// * `BRC20_PROG_RPC_SERVER_ENABLE_AUTH` - Whether to enable authentication for the BRC20 Prog RPC server (Default: false)
    /// * `BRC20_PROG_RPC_SERVER_USER` - The username for the BRC20 Prog RPC server, if authentication is enabled
    /// * `BRC20_PROG_RPC_SERVER_PASSWORD` - The password for the BRC20 Prog RPC server, if authentication is enabled
    /// * `BRC20_PROG_BALANCE_SERVER_URL` - The URL of the BRC20 balance server (Default: "http://localhost:18546")
    /// * `EVM_RECORD_TRACES` - Whether to record EVM traces (Default: false)
    /// * `BITCOIN_RPC_URL` - The URL of the Bitcoin RPC server (Default: "http://localhost:38332" for signet)
    /// * `BITCOIN_RPC_USER` - The username for the Bitcoin RPC server
    /// * `BITCOIN_RPC_PASSWORD` - The password for the Bitcoin RPC server
    /// * `BITCOIN_RPC_NETWORK` - The network type for the Bitcoin RPC server (Default: "signet")
    /// * `FAIL_ON_BRC20_BALANCE_SERVER_ERROR` - Whether to fail on BRC20 balance server errors (Default: true)
    /// * `FAIL_ON_BITCOIN_RPC_ERROR` - Whether to fail on Bitcoin RPC errors (Default: true)
    pub fn from_env() -> Self {
        Self {
            brc20_prog_rpc_server_url: env::var(&*BRC20_PROG_RPC_SERVER_URL_KEY)
                .unwrap_or(BRC20_PROG_RPC_SERVER_URL_DEFAULT.clone()),
            brc20_prog_rpc_server_enable_auth: env::var(&*BRC20_PROG_RPC_SERVER_ENABLE_AUTH_KEY)
                .map(|x| x == "true")
                .unwrap_or(*BRC20_PROG_RPC_SERVER_ENABLE_AUTH_DEFAULT),
            brc20_prog_rpc_server_user: env::var(&*BRC20_PROG_RPC_SERVER_USER_KEY).ok(),
            brc20_prog_rpc_server_password: env::var(&*BRC20_PROG_RPC_SERVER_PASSWORD_KEY).ok(),
            brc20_balance_server_url: env::var(&*BRC20_PROG_BALANCE_SERVER_URL_KEY)
                .unwrap_or(BRC20_PROG_BALANCE_SERVER_URL_DEFAULT.clone()),
            evm_record_traces: env::var(&*EVM_RECORD_TRACES_KEY)
                .map(|x| x == "true")
                .unwrap_or(*EVM_RECORD_TRACES_DEFAULT),
            evm_call_gas_limit: env::var(&*EVM_CALL_GAS_LIMIT_KEY)
                .map(|x| x.parse::<u64>().unwrap_or(*EVM_CALL_GAS_LIMIT))
                .unwrap_or(*EVM_CALL_GAS_LIMIT),
            bitcoin_rpc_url: env::var(&*BITCOIN_RPC_URL_KEY)
                .unwrap_or(BITCOIN_RPC_URL_DEFAULT_SIGNET.clone()),
            bitcoin_rpc_user: env::var(&*BITCOIN_RPC_USER_KEY).unwrap_or(Default::default()),
            bitcoin_rpc_password: env::var(&*BITCOIN_RPC_PASSWORD_KEY)
                .unwrap_or(Default::default()),
            bitcoin_rpc_network: env::var(&*BITCOIN_RPC_NETWORK_KEY)
                .unwrap_or("signet".to_string()),
            chain_id: if env::var(&*BITCOIN_RPC_NETWORK_KEY).unwrap_or("signet".to_string())
                == "mainnet"
            {
                CHAIN_ID
            } else {
                CHAIN_ID_TESTNETS
            },
            fail_on_brc20_balance_server_error: env::var(&*FAIL_ON_BRC20_BALANCE_SERVER_ERROR_KEY)
                .map(|x| x == "true")
                .unwrap_or(*FAIL_ON_BRC20_BALANCE_SERVER_ERROR_DEFAULT),
            fail_on_bitcoin_rpc_error: env::var(&*FAIL_ON_BITCOIN_RPC_ERROR_KEY)
                .map(|x| x == "true")
                .unwrap_or(*FAIL_ON_BITCOIN_RPC_ERROR_DEFAULT),
            db_path: env::var(&*DB_PATH_KEY).unwrap_or(DB_PATH_DEFAULT.clone()),
        }
    }
}

#[cfg(feature = "server")]
pub fn validate_config(config: &Brc20ProgConfig) -> Result<(), Box<dyn std::error::Error>> {
    if config.brc20_prog_rpc_server_enable_auth
        && (config.brc20_prog_rpc_server_user.is_none()
            || config.brc20_prog_rpc_server_password.is_none())
    {
        return Err("Authentication is enabled but no username or password is set".into());
    }

    if config.brc20_prog_rpc_server_url.is_empty() {
        return Err("RPC server URL is empty".into());
    }

    if config.brc20_balance_server_url.is_empty() && config.fail_on_brc20_balance_server_error {
        return Err("BRC20 balance server URL is empty".into());
    }

    if !config.brc20_balance_server_url.starts_with("http://")
        && !config.brc20_balance_server_url.starts_with("https://")
        && config.fail_on_brc20_balance_server_error
    {
        return Err("BRC20 balance server URL must start with http:// or https://".into());
    }

    if config.bitcoin_rpc_url.is_empty() && config.fail_on_bitcoin_rpc_error {
        return Err("Bitcoin RPC URL is empty".into());
    }

    if config.bitcoin_rpc_user.is_empty() && config.fail_on_bitcoin_rpc_error {
        return Err("Bitcoin RPC user is empty".into());
    }

    if config.bitcoin_rpc_password.is_empty() && config.fail_on_bitcoin_rpc_error {
        return Err("Bitcoin RPC password is empty".into());
    }

    if config.bitcoin_rpc_network.is_empty() && config.fail_on_bitcoin_rpc_error {
        return Err("Bitcoin RPC network is empty".into());
    }

    Ok(())
}
