use std::env;

use alloy::primitives::Address;

use crate::global::shared_data::SharedData;

lazy_static::lazy_static! {
    pub(crate) static ref DB_VERSION_KEY: String = "DB_VERSION".to_string();
    pub(crate) static ref DB_VERSION: u32 = 7;

    pub(crate) static ref PROTOCOL_VERSION_KEY: String = "PROTOCOL_VERSION".to_string();
    pub(crate) static ref PROTOCOL_VERSION: u32 = 2;

    static ref DB_PATH_KEY: String = "BRC20_PROG_DB_PATH".to_string();
    static ref DB_PATH_DEFAULT: String = "target/db".to_string();

    static ref BRC20_PROG_RPC_SERVER_ENABLE_AUTH_KEY: String = "BRC20_PROG_RPC_SERVER_ENABLE_AUTH".to_string();
    static ref BRC20_PROG_RPC_SERVER_ENABLE_AUTH_DEFAULT: bool = true;

    static ref BRC20_PROG_RPC_SERVER_USER_KEY: String = "BRC20_PROG_RPC_SERVER_USER".to_string();
    static ref BRC20_PROG_RPC_SERVER_PASSWORD_KEY: String = "BRC20_PROG_RPC_SERVER_PASSWORD".to_string();

    static ref BRC20_PROG_RPC_SERVER_URL_KEY: String = "BRC20_PROG_RPC_SERVER_URL".to_string();
    static ref BRC20_PROG_RPC_SERVER_URL_DEFAULT: String = "127.0.0.1:18545".to_string();

    static ref BRC20_PROG_ADMIN_RPC_SERVER_URL_KEY: String = "BRC20_PROG_ADMIN_RPC_SERVER_URL".to_string();

    static ref BRC20_PROG_RPC_TRUST_FORWARDED_FOR_KEY: String = "BRC20_PROG_RPC_TRUST_FORWARDED_FOR".to_string();
    static ref BRC20_PROG_RPC_TRUST_FORWARDED_FOR_DEFAULT: bool = false;

    static ref BRC20_PROG_RPC_TX_POOL_KEY: String = "BRC20_PROG_RPC_TX_POOL".to_string();
    static ref BRC20_PROG_RPC_TX_POOL_DEFAULT: bool = false;

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

    static ref MAX_REQUEST_SIZE_KEY: String = "MAX_REQUEST_SIZE".to_string();
    static ref MAX_REQUEST_SIZE_DEFAULT: u32 = 10 * 1024 * 1024; // 10MB

    static ref MAX_RESPONSE_SIZE_KEY: String = "MAX_RESPONSE_SIZE".to_string();
    static ref MAX_RESPONSE_SIZE_DEFAULT: u32 = 100 * 1024 * 1024; // 100MB

    static ref BATCH_REQUEST_LIMIT_KEY: String = "BATCH_REQUEST_LIMIT".to_string();
    static ref BATCH_REQUEST_LIMIT_DEFAULT: u32 = 50; // 50 requests

    static ref GET_LOGS_MAX_BLOCKS_KEY: String = "GET_LOGS_MAX_BLOCKS".to_string();
    static ref GET_LOGS_MAX_BLOCKS_DEFAULT: u64 = 6;

    pub static ref CARGO_PKG_VERSION: String = {
        let version = env!("CARGO_PKG_VERSION");
        if version.is_empty() {
            "0.0.0".to_string()
        } else {
            version.to_string()
        }
    };

    pub static ref CARGO_RUST_VERSION: String = {
        let version = env!("CARGO_PKG_RUST_VERSION");
        if version.is_empty() {
            "unknown".to_string()
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

pub const GAS_PER_OP_RETURN_TX_ID: u64 = 40; // 40 gas for fetching the op return tx id
pub const GAS_PER_BITCOIN_RPC_CALL: u64 = 400000; // 400K gas per Bitcoin RPC call
pub const GAS_PER_BIP_322_VERIFY: u64 = 20000; // 20K gas per BIP-322 verify call
pub const GAS_PER_LOCKED_PKSCRIPT: u64 = 20000; // 20K gas per locked pkscript call

const CHAIN_ID: u64 = 0x4252433230; // Mainnet Chain ID: BRC20 in hex
const CHAIN_ID_TESTNETS: u64 = 0x425243323073; // Testnets Chain ID: BRC20s in hex

pub const CALLDATA_LIMIT: usize = 1024 * 1024; // 1MB

/// Configuration for the BRC20 Prog server
/// This struct holds the configuration values for the BRC20 Prog server.
///
/// Its `Debug` output redacts both passwords.
#[derive(Clone)]
pub struct Brc20ProgConfig {
    /// The URL of the BRC20 Prog RPC server
    pub brc20_prog_rpc_server_url: String,
    /// The URL of the admin RPC server, which serves the indexer methods.
    /// When set, the public server rejects indexer methods regardless of authentication.
    /// When unset, a single server serves every method and HTTP basic auth gates the indexer
    /// methods.
    pub brc20_prog_admin_rpc_server_url: Option<String>,
    /// Must be true. [`start`](crate::start) refuses to start when it is false, since the
    /// JSON-RPC server always requires HTTP basic auth for the indexer methods.
    /// [`Brc20Prog::open`](crate::Brc20Prog::open) ignores it.
    pub brc20_prog_rpc_server_enable_auth: bool,
    /// The HTTP basic auth username that guards the indexer methods. Required by
    /// [`start`](crate::start), unused in library mode.
    pub brc20_prog_rpc_server_user: Option<String>,
    /// The HTTP basic auth password that guards the indexer methods. Required by
    /// [`start`](crate::start), which rejects a weak one (shorter than 32 characters, a sample
    /// value, equal to the username, or one repeated character). Unused in library mode.
    pub brc20_prog_rpc_server_password: Option<String>,
    /// Whether the first `X-Forwarded-For` hop identifies the client for rate limiting.
    /// Enable it only behind a proxy that replaces the client's `X-Forwarded-For` header,
    /// since clients can forge it.
    /// When disabled, every caller shares one rate limit window.
    pub brc20_prog_rpc_trust_forwarded_for: bool,
    /// Whether the JSON-RPC server serves the pending raw transaction pool:
    /// `eth_sendRawTransaction`, `eth_getRawTransactionByHash` and
    /// `brc20_unsentRawTransactions`, a pool-aware `eth_getTransactionCount(addr, "pending")`,
    /// and rate limits on the pool methods.
    /// When disabled, the pool methods return JSON-RPC "method not found".
    pub brc20_prog_rpc_tx_pool: bool,
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
    /// Whether to fail on Bitcoin RPC errors, if set to true, the server will stop if Bitcoin RPC server is not reachable when needed
    pub fail_on_bitcoin_rpc_error: bool,
    /// Database path
    pub db_path: String,

    /// Max request size in bytes
    pub max_request_size: u32,
    /// Max response size in bytes
    pub max_response_size: u32,
    /// Batch request limit (0 for unlimited)
    pub batch_request_limit: u32,
    /// Maximum number of blocks a single eth_getLogs call may span (inclusive)
    pub get_logs_max_blocks: u64,
}

impl std::fmt::Debug for Brc20ProgConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const REDACTED: &str = "<redacted>";
        f.debug_struct("Brc20ProgConfig")
            .field("brc20_prog_rpc_server_url", &self.brc20_prog_rpc_server_url)
            .field(
                "brc20_prog_admin_rpc_server_url",
                &self.brc20_prog_admin_rpc_server_url,
            )
            .field(
                "brc20_prog_rpc_server_enable_auth",
                &self.brc20_prog_rpc_server_enable_auth,
            )
            .field(
                "brc20_prog_rpc_server_user",
                &self.brc20_prog_rpc_server_user,
            )
            .field(
                "brc20_prog_rpc_server_password",
                &self
                    .brc20_prog_rpc_server_password
                    .as_ref()
                    .map(|_| REDACTED),
            )
            .field("evm_record_traces", &self.evm_record_traces)
            .field("evm_call_gas_limit", &self.evm_call_gas_limit)
            .field("bitcoin_rpc_url", &self.bitcoin_rpc_url)
            .field("bitcoin_rpc_user", &self.bitcoin_rpc_user)
            .field("bitcoin_rpc_password", &REDACTED)
            .field("bitcoin_rpc_network", &self.bitcoin_rpc_network)
            .field("chain_id", &self.chain_id)
            .field("fail_on_bitcoin_rpc_error", &self.fail_on_bitcoin_rpc_error)
            .field("db_path", &self.db_path)
            .field("max_request_size", &self.max_request_size)
            .field("max_response_size", &self.max_response_size)
            .field("batch_request_limit", &self.batch_request_limit)
            .field("get_logs_max_blocks", &self.get_logs_max_blocks)
            .finish()
    }
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
    /// * `brc20_prog_rpc_server_enable_auth` - Must be true for [`start`](crate::start); false is rejected
    /// * `brc20_prog_rpc_server_user` - HTTP basic auth username for the indexer methods, required by [`start`](crate::start)
    /// * `brc20_prog_rpc_server_password` - HTTP basic auth password for the indexer methods, required by [`start`](crate::start)
    /// * `evm_call_gas_limit` - Gas limit for EVM calls (default: 1_000_000_000)
    /// * `evm_record_traces` - Whether to record EVM traces
    /// * `bitcoin_rpc_url` - The URL of the Bitcoin RPC server
    /// * `bitcoin_rpc_user` - The username for the Bitcoin RPC server
    /// * `bitcoin_rpc_password` - The password for the Bitcoin RPC server
    /// * `bitcoin_rpc_network` - The network type for the Bitcoin RPC server
    /// * `fail_on_bitcoin_rpc_error` - Whether to fail on Bitcoin RPC errors
    /// * `db_path` - The path to the database folder
    /// * `max_request_size` - Max request size in bytes
    /// * `max_response_size` - Max response size in bytes
    /// * `batch_request_limit` - Batch request limit (0 for unlimited)
    pub fn new(
        brc20_prog_rpc_server_url: String,
        brc20_prog_rpc_server_enable_auth: bool,
        brc20_prog_rpc_server_user: Option<String>,
        brc20_prog_rpc_server_password: Option<String>,
        evm_record_traces: bool,
        evm_call_gas_limit: u64,
        bitcoin_rpc_url: String,
        bitcoin_rpc_user: String,
        bitcoin_rpc_password: String,
        bitcoin_rpc_network: String,
        chain_id: u64,
        fail_on_bitcoin_rpc_error: bool,
        db_path: String,
        max_request_size: u32,
        max_response_size: u32,
        batch_request_limit: u32,
    ) -> Self {
        Self {
            brc20_prog_rpc_server_url,
            brc20_prog_admin_rpc_server_url: None,
            brc20_prog_rpc_server_enable_auth,
            brc20_prog_rpc_server_user,
            brc20_prog_rpc_server_password,
            brc20_prog_rpc_trust_forwarded_for: *BRC20_PROG_RPC_TRUST_FORWARDED_FOR_DEFAULT,
            brc20_prog_rpc_tx_pool: *BRC20_PROG_RPC_TX_POOL_DEFAULT,
            evm_record_traces,
            evm_call_gas_limit,
            bitcoin_rpc_url,
            bitcoin_rpc_user,
            bitcoin_rpc_password,
            bitcoin_rpc_network,
            chain_id,
            fail_on_bitcoin_rpc_error,
            db_path,
            max_request_size,
            max_response_size,
            batch_request_limit,
            get_logs_max_blocks: *GET_LOGS_MAX_BLOCKS_DEFAULT,
        }
    }

    /// Creates a new instance of `Brc20ProgConfig` from environment variables
    ///
    /// This function reads the configuration values from environment variables and returns a new instance of `Brc20ProgConfig`.
    ///
    /// List of environment variables read:
    /// * `BRC20_PROG_DB_PATH` - The path to the database folder (Default: "target/db")
    /// * `BRC20_PROG_RPC_SERVER_URL` - The URL of the BRC20 Prog RPC server (Default: "127.0.0.1:18545")
    /// * `BRC20_PROG_ADMIN_RPC_SERVER_URL` - The URL of the admin RPC server that serves the indexer methods (Default: unset, single server)
    /// * `BRC20_PROG_RPC_SERVER_ENABLE_AUTH` - Retired; `false` makes [`start`](crate::start) fail, any other value is ignored (Default: true)
    /// * `BRC20_PROG_RPC_SERVER_USER` - HTTP basic auth username for the indexer methods, required by [`start`](crate::start)
    /// * `BRC20_PROG_RPC_SERVER_PASSWORD` - HTTP basic auth password for the indexer methods, required by [`start`](crate::start)
    /// * `BRC20_PROG_RPC_TRUST_FORWARDED_FOR` - Whether the first `X-Forwarded-For` hop identifies the client for rate limiting (Default: false)
    /// * `BRC20_PROG_RPC_TX_POOL` - Whether the JSON-RPC server serves the pending raw transaction pool (Default: false)
    /// * `EVM_RECORD_TRACES` - Whether to record EVM traces (Default: false)
    /// * `BITCOIN_RPC_URL` - The URL of the Bitcoin RPC server (Default: "http://localhost:38332" for signet)
    /// * `BITCOIN_RPC_USER` - The username for the Bitcoin RPC server
    /// * `BITCOIN_RPC_PASSWORD` - The password for the Bitcoin RPC server
    /// * `BITCOIN_RPC_NETWORK` - The network type for the Bitcoin RPC server (Default: "signet")
    /// * `FAIL_ON_BITCOIN_RPC_ERROR` - Whether to fail on Bitcoin RPC errors (Default: true)
    /// * `EVM_CALL_GAS_LIMIT` - Gas limit for EVM calls (Default: 1_000_000_000)
    /// * `MAX_REQUEST_SIZE` - Max request size in bytes (Default: 10MB)
    /// * `MAX_RESPONSE_SIZE` - Max response size in bytes (Default: 100MB)
    /// * `BATCH_REQUEST_LIMIT` - Batch request limit (0 for unlimited) (Default: 50)
    /// * `GET_LOGS_MAX_BLOCKS` - Maximum blocks per eth_getLogs call (Default: 6)
    /// # Returns
    /// A new instance of `Brc20ProgConfig` with the configuration values read from environment variables.
    pub fn from_env() -> Self {
        let bitcoin_rpc_network =
            env::var(&*BITCOIN_RPC_NETWORK_KEY).unwrap_or("signet".to_string());

        let chain_id = if bitcoin_rpc_network == "bitcoin" || bitcoin_rpc_network == "mainnet" {
            CHAIN_ID
        } else {
            CHAIN_ID_TESTNETS
        };

        Self {
            brc20_prog_rpc_server_url: env::var(&*BRC20_PROG_RPC_SERVER_URL_KEY)
                .unwrap_or(BRC20_PROG_RPC_SERVER_URL_DEFAULT.clone()),
            brc20_prog_admin_rpc_server_url: env::var(&*BRC20_PROG_ADMIN_RPC_SERVER_URL_KEY)
                .ok()
                .filter(|x| !x.is_empty()),
            brc20_prog_rpc_server_enable_auth: env::var(&*BRC20_PROG_RPC_SERVER_ENABLE_AUTH_KEY)
                .map(|x| !x.trim().eq_ignore_ascii_case("false"))
                .unwrap_or(*BRC20_PROG_RPC_SERVER_ENABLE_AUTH_DEFAULT),
            brc20_prog_rpc_server_user: env::var(&*BRC20_PROG_RPC_SERVER_USER_KEY).ok(),
            brc20_prog_rpc_server_password: env::var(&*BRC20_PROG_RPC_SERVER_PASSWORD_KEY).ok(),
            brc20_prog_rpc_trust_forwarded_for: env::var(&*BRC20_PROG_RPC_TRUST_FORWARDED_FOR_KEY)
                .map(|x| x == "true")
                .unwrap_or(*BRC20_PROG_RPC_TRUST_FORWARDED_FOR_DEFAULT),
            brc20_prog_rpc_tx_pool: env::var(&*BRC20_PROG_RPC_TX_POOL_KEY)
                .map(|x| x == "true")
                .unwrap_or(*BRC20_PROG_RPC_TX_POOL_DEFAULT),
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
            chain_id,
            fail_on_bitcoin_rpc_error: env::var(&*FAIL_ON_BITCOIN_RPC_ERROR_KEY)
                .map(|x| x == "true")
                .unwrap_or(*FAIL_ON_BITCOIN_RPC_ERROR_DEFAULT),
            db_path: env::var(&*DB_PATH_KEY).unwrap_or(DB_PATH_DEFAULT.clone()),

            max_request_size: env::var(&*MAX_REQUEST_SIZE_KEY)
                .map(|x| x.parse::<u32>().unwrap_or(*MAX_REQUEST_SIZE_DEFAULT))
                .unwrap_or(*MAX_REQUEST_SIZE_DEFAULT),
            max_response_size: env::var(&*MAX_RESPONSE_SIZE_KEY)
                .map(|x| x.parse::<u32>().unwrap_or(*MAX_RESPONSE_SIZE_DEFAULT))
                .unwrap_or(*MAX_RESPONSE_SIZE_DEFAULT),
            batch_request_limit: env::var(&*BATCH_REQUEST_LIMIT_KEY)
                .map(|x| x.parse::<u32>().unwrap_or(*BATCH_REQUEST_LIMIT_DEFAULT))
                .unwrap_or(*BATCH_REQUEST_LIMIT_DEFAULT),
            get_logs_max_blocks: env::var(&*GET_LOGS_MAX_BLOCKS_KEY)
                .ok()
                .and_then(|x| x.parse::<u64>().ok())
                .filter(|x| *x > 0)
                .unwrap_or(*GET_LOGS_MAX_BLOCKS_DEFAULT),
        }
    }
}

/// Minimum length of the password that guards the indexer methods.
#[cfg(feature = "server")]
pub const MIN_RPC_PASSWORD_LENGTH: usize = 32;

/// Sample and default values that are never accepted as the indexer methods' password.
#[cfg(feature = "server")]
const WEAK_RPC_PASSWORDS: [&str; 6] = ["password", "changeme", "admin", "secret", "user", "test"];

#[cfg(feature = "server")]
const GENERATE_PASSWORD_HINT: &str = "generate one with `openssl rand -hex 32`";

/// Checks that `user` and `password` are strong enough to guard the indexer methods.
///
/// Rejects an empty username or password, a known weak or sample password (case-insensitive), a
/// password equal to the username, a password of one repeated character, and a password shorter
/// than [`MIN_RPC_PASSWORD_LENGTH`]. Error messages never contain the credentials.
#[cfg(feature = "server")]
pub fn validate_rpc_credentials(user: &str, password: &str) -> Result<(), String> {
    if user.is_empty() {
        return Err("BRC20_PROG_RPC_SERVER_USER is empty".to_string());
    }
    if password.is_empty() {
        return Err(format!(
            "BRC20_PROG_RPC_SERVER_PASSWORD is empty, {}",
            GENERATE_PASSWORD_HINT
        ));
    }
    if WEAK_RPC_PASSWORDS
        .iter()
        .any(|weak| password.eq_ignore_ascii_case(weak))
    {
        return Err(format!(
            "BRC20_PROG_RPC_SERVER_PASSWORD is a known weak or sample value, {}",
            GENERATE_PASSWORD_HINT
        ));
    }
    if password.eq_ignore_ascii_case(user) {
        return Err(format!(
            "BRC20_PROG_RPC_SERVER_PASSWORD equals the username, {}",
            GENERATE_PASSWORD_HINT
        ));
    }
    let mut chars = password.chars();
    let first = chars.next();
    if chars.all(|c| Some(c) == first) {
        return Err(format!(
            "BRC20_PROG_RPC_SERVER_PASSWORD repeats a single character, {}",
            GENERATE_PASSWORD_HINT
        ));
    }
    if password.chars().count() < MIN_RPC_PASSWORD_LENGTH {
        return Err(format!(
            "BRC20_PROG_RPC_SERVER_PASSWORD is shorter than {} characters, {}",
            MIN_RPC_PASSWORD_LENGTH, GENERATE_PASSWORD_HINT
        ));
    }
    Ok(())
}

/// Returns the credentials that guard the indexer methods when the server serves them over
/// JSON-RPC, after checking them with [`validate_rpc_credentials`].
///
/// Fails when `brc20_prog_rpc_server_enable_auth` is false, since the server never serves the
/// indexer methods without authentication, or when the credentials are missing or weak.
#[cfg(feature = "server")]
pub fn validate_rpc_server_credentials(
    config: &Brc20ProgConfig,
) -> Result<(&str, &str), Box<dyn std::error::Error>> {
    if !config.brc20_prog_rpc_server_enable_auth {
        return Err(
            "BRC20_PROG_RPC_SERVER_ENABLE_AUTH=false is not supported: the JSON-RPC server always \
             requires HTTP basic auth for the indexer methods. Set BRC20_PROG_RPC_SERVER_USER and \
             BRC20_PROG_RPC_SERVER_PASSWORD, or embed the module as a library (Brc20Prog::open) \
             and serve only the public methods with Brc20Prog::start_public_rpc_server"
                .into(),
        );
    }
    let Some(user) = config.brc20_prog_rpc_server_user.as_deref() else {
        return Err(
            "BRC20_PROG_RPC_SERVER_USER is required: the JSON-RPC server always requires HTTP \
             basic auth for the indexer methods"
                .into(),
        );
    };
    let Some(password) = config.brc20_prog_rpc_server_password.as_deref() else {
        return Err(format!(
            "BRC20_PROG_RPC_SERVER_PASSWORD is required: the JSON-RPC server always requires \
             HTTP basic auth for the indexer methods, {}",
            GENERATE_PASSWORD_HINT
        )
        .into());
    };
    validate_rpc_credentials(user, password)?;
    Ok((user, password))
}

#[cfg(feature = "server")]
pub fn validate_config(config: &Brc20ProgConfig) -> Result<(), Box<dyn std::error::Error>> {
    if config.brc20_prog_rpc_server_url.is_empty() {
        return Err("RPC server URL is empty".into());
    }

    if let Some(admin_url) = &config.brc20_prog_admin_rpc_server_url {
        if admin_url.parse::<std::net::SocketAddr>().is_err() {
            return Err("Admin RPC server URL is not a valid socket address".into());
        }
        if *admin_url == config.brc20_prog_rpc_server_url {
            return Err("Admin RPC server URL must differ from the public RPC server URL".into());
        }
    }

    if config.bitcoin_rpc_network.is_empty() && config.fail_on_bitcoin_rpc_error {
        return Err("Bitcoin RPC network is empty".into());
    }

    Ok(())
}

/// Checks the settings the built-in Bitcoin Core client needs. An injected
/// [`BitcoinRpc`](crate::BitcoinRpc) source does not use them.
#[cfg(feature = "server")]
pub fn validate_bitcoin_core_config(
    config: &Brc20ProgConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if config.bitcoin_rpc_url.is_empty() && config.fail_on_bitcoin_rpc_error {
        return Err("Bitcoin RPC URL is empty".into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "server")]
    fn valid_config() -> Brc20ProgConfig {
        Brc20ProgConfig::new(
            "127.0.0.1:18545".to_string(),        // rpc url
            false,                                // enable auth
            None,                                 // user
            None,                                 // password
            false,                                // record traces
            1_000_000_000,                        // call gas limit
            "http://localhost:38332".to_string(), // bitcoin rpc url
            "user".to_string(),
            "pass".to_string(),
            "signet".to_string(), // network
            CHAIN_ID_TESTNETS,
            true, // fail_on_bitcoin_rpc_error
            "target/db".to_string(),
            10 * 1024 * 1024,
            100 * 1024 * 1024,
            50,
        )
    }

    #[cfg(feature = "server")]
    #[test]
    fn test_validate_config_accepts_valid() {
        assert!(validate_config(&valid_config()).is_ok());
    }

    #[cfg(feature = "server")]
    const STRONG_PASSWORD: &str = "3f9c2a7e5b1d48c6a0e7f2b9d4c1a8e6";

    #[cfg(feature = "server")]
    #[test]
    fn test_validate_config_does_not_need_credentials() {
        // Library mode opens without RPC credentials, whatever the auth flag says.
        let mut config = valid_config();
        assert!(validate_config(&config).is_ok());
        config.brc20_prog_rpc_server_enable_auth = true;
        assert!(validate_config(&config).is_ok());
    }

    #[cfg(feature = "server")]
    fn assert_rejected(user: &str, password: &str, reason: &str) {
        let error = validate_rpc_credentials(user, password).unwrap_err();
        assert!(
            error.contains(reason),
            "{error:?} should mention {reason:?}"
        );
        if !password.is_empty() {
            assert!(!error.contains(password), "error leaks the password");
        }
    }

    #[cfg(feature = "server")]
    #[test]
    fn test_validate_rpc_credentials_accepts_strong_password() {
        assert!(validate_rpc_credentials("indexer", STRONG_PASSWORD).is_ok());
    }

    #[cfg(feature = "server")]
    #[test]
    fn test_validate_rpc_credentials_rejects_empty() {
        assert_rejected("", STRONG_PASSWORD, "USER is empty");
        assert_rejected("indexer", "", "PASSWORD is empty");
    }

    #[cfg(feature = "server")]
    #[test]
    fn test_validate_rpc_credentials_rejects_short_password() {
        let short = &STRONG_PASSWORD[..MIN_RPC_PASSWORD_LENGTH - 1];
        assert_rejected("indexer", short, "shorter than 32");
        assert!(validate_rpc_credentials("indexer", &STRONG_PASSWORD[..32]).is_ok());
    }

    #[cfg(feature = "server")]
    #[test]
    fn test_validate_rpc_credentials_rejects_password_equal_to_user() {
        assert_rejected(STRONG_PASSWORD, STRONG_PASSWORD, "equals the username");
        assert_rejected(
            &STRONG_PASSWORD.to_uppercase(),
            STRONG_PASSWORD,
            "equals the username",
        );
    }

    #[cfg(feature = "server")]
    #[test]
    fn test_validate_rpc_credentials_rejects_weak_values() {
        for weak in [
            "password", "PassWord", "Password", "changeme", "ChangeMe", "admin", "secret", "user",
            "test", "TEST",
        ] {
            assert_rejected("indexer", weak, "known weak or sample value");
        }
    }

    #[cfg(feature = "server")]
    #[test]
    fn test_validate_rpc_credentials_rejects_repeated_character() {
        assert_rejected("indexer", &"a".repeat(64), "repeats a single character");
        assert_rejected("indexer", "zz", "repeats a single character");
    }

    #[cfg(feature = "server")]
    #[test]
    fn test_validate_rpc_credentials_error_suggests_generator() {
        let error = validate_rpc_credentials("indexer", "short").unwrap_err();
        assert!(error.contains("openssl rand -hex 32"));
    }

    #[cfg(feature = "server")]
    #[test]
    fn test_validate_rpc_server_credentials() {
        let mut config = valid_config();
        config.brc20_prog_rpc_server_enable_auth = true;

        // Missing credentials
        assert!(validate_rpc_server_credentials(&config).is_err());
        config.brc20_prog_rpc_server_user = Some("indexer".to_string());
        assert!(validate_rpc_server_credentials(&config).is_err());

        // Weak password, and the error does not leak it
        config.brc20_prog_rpc_server_password = Some("password".to_string());
        let error = validate_rpc_server_credentials(&config)
            .unwrap_err()
            .to_string();
        assert!(!error.contains("password") && error.contains("weak"));

        // Strong password
        config.brc20_prog_rpc_server_password = Some(STRONG_PASSWORD.to_string());
        assert_eq!(
            validate_rpc_server_credentials(&config).unwrap(),
            ("indexer", STRONG_PASSWORD)
        );

        // Auth explicitly disabled is rejected even with strong credentials
        config.brc20_prog_rpc_server_enable_auth = false;
        let error = validate_rpc_server_credentials(&config)
            .unwrap_err()
            .to_string();
        assert!(error.contains("ENABLE_AUTH=false"));
        assert!(error.contains("start_public_rpc_server"));
    }

    #[test]
    fn test_debug_redacts_passwords() {
        let config = Brc20ProgConfig {
            brc20_prog_rpc_server_user: Some("indexer".to_string()),
            brc20_prog_rpc_server_password: Some("rpc-secret-value".to_string()),
            bitcoin_rpc_password: "btc-secret-value".to_string(),
            ..Brc20ProgConfig::from_env()
        };
        let debug = format!("{config:?}");
        assert!(!debug.contains("rpc-secret-value"));
        assert!(!debug.contains("btc-secret-value"));
        assert!(debug.contains("indexer"));
    }

    #[test]
    fn test_from_env_enable_auth() {
        let key = &*BRC20_PROG_RPC_SERVER_ENABLE_AUTH_KEY;
        let _guard = EnvVarGuard::set(key, "false");
        assert!(!Brc20ProgConfig::from_env().brc20_prog_rpc_server_enable_auth);

        env::set_var(key, "FALSE");
        assert!(!Brc20ProgConfig::from_env().brc20_prog_rpc_server_enable_auth);

        env::set_var(key, "true");
        assert!(Brc20ProgConfig::from_env().brc20_prog_rpc_server_enable_auth);

        env::remove_var(key);
        assert!(Brc20ProgConfig::from_env().brc20_prog_rpc_server_enable_auth);
    }

    #[cfg(feature = "server")]
    #[test]
    fn test_validate_config_rejects_empty_rpc_url() {
        let mut config = valid_config();
        config.brc20_prog_rpc_server_url = String::new();
        assert!(validate_config(&config).is_err());
    }

    #[cfg(feature = "server")]
    #[test]
    fn test_validate_config_admin_url() {
        let mut config = valid_config();

        config.brc20_prog_admin_rpc_server_url = Some("127.0.0.1:18546".to_string());
        assert!(validate_config(&config).is_ok());

        config.brc20_prog_admin_rpc_server_url = Some("not-an-address".to_string());
        assert!(validate_config(&config).is_err());

        config.brc20_prog_admin_rpc_server_url = Some(config.brc20_prog_rpc_server_url.clone());
        assert!(validate_config(&config).is_err());
    }

    #[cfg(feature = "server")]
    #[test]
    fn test_validate_config_bitcoin_url_only_required_when_failing_on_rpc_error() {
        let mut config = valid_config();
        config.bitcoin_rpc_url = String::new();

        // Empty Bitcoin URL is fatal only when we fail on Bitcoin RPC errors.
        config.fail_on_bitcoin_rpc_error = true;
        assert!(validate_config(&config).is_ok());
        assert!(validate_bitcoin_core_config(&config).is_err());

        config.fail_on_bitcoin_rpc_error = false;
        assert!(validate_bitcoin_core_config(&config).is_ok());
    }

    // Restores a process-global env var to its previous value on drop, so these
    // tests stay isolated and panic-safe (cleanup runs even if an assert fails).
    struct EnvVarGuard {
        key: String,
        previous: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &str, value: &str) -> Self {
            let previous = env::var(key).ok();
            env::set_var(key, value);
            EnvVarGuard {
                key: key.to_string(),
                previous,
            }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(value) => env::set_var(&self.key, value),
                None => env::remove_var(&self.key),
            }
        }
    }

    #[test]
    fn test_from_env_selects_chain_id_by_network() {
        let key = &*BITCOIN_RPC_NETWORK_KEY;
        // Guard captures the original value once; it is restored on drop.
        let _guard = EnvVarGuard::set(key, "bitcoin");
        assert_eq!(Brc20ProgConfig::from_env().chain_id, CHAIN_ID);

        env::set_var(key, "mainnet");
        assert_eq!(Brc20ProgConfig::from_env().chain_id, CHAIN_ID);

        env::set_var(key, "signet");
        assert_eq!(Brc20ProgConfig::from_env().chain_id, CHAIN_ID_TESTNETS);
    }

    #[test]
    fn test_from_env_falls_back_on_malformed_numeric() {
        let key = &*EVM_CALL_GAS_LIMIT_KEY;
        let _guard = EnvVarGuard::set(key, "not-a-number");
        assert_eq!(
            Brc20ProgConfig::from_env().evm_call_gas_limit,
            *EVM_CALL_GAS_LIMIT
        );
    }
}
