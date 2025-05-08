use std::error::Error;
use std::path::Path;

use jsonrpsee::server::ServerHandle;
use tracing::{error, info, warn};

use crate::db::DB;
use crate::evm::precompiles::validate_bitcoin_rpc_status;
use crate::global::{validate_config, Brc20ProgConfig, CONFIG};
use crate::server::{start_rpc_server, BRC20ProgEngine};

/// Starts the BRC20 programmable module server.
///
/// This function initializes the logging, validates the configuration, checks the Bitcoin RPC status,
/// initializes the database, and starts the JSON-RPC server.
///
/// Server can be configured by passing a `Brc20ProgConfig` instance.
///
/// If authentication is enabled, following JSON-RPC methods will require authentication, and the rest will be accessible
/// without authentication:
/// * brc20_mine
/// * brc20_deploy
/// * brc20_call
/// * brc20_deposit
/// * brc20_withdraw
/// * brc20_initialise
/// * brc20_finaliseBlock
/// * brc20_reorg
/// * brc20_commitToDatabase
/// * brc20_clearCaches
///
/// # Errors
///
/// This function will return an error if:
/// * The logging initialization fails.
/// * The configuration validation fails.
/// * The Bitcoin RPC status check fails and `FAIL_ON_BITCOIN_RPC_ERROR` is true.
/// * The database initialization fails.
/// * The JSON-RPC server fails to start.
///
/// # Example
///
/// Below is an example on how to use the `start` function:
///
/// ```
/// use std::error::Error;
/// use brc20_prog::{Brc20ProgConfig, start};
///
/// pub async fn interact_with_server() -> Result<(), Box<dyn Error>> {
///     let server_handle = start(Brc20ProgConfig::from_env()).await?;
///     // Do something with the server handle, e.g. send requests to the server, or use the client.
///     // ...
///     // When done, stop the server
///     server_handle.stop()?;
///     Ok(())
/// }
/// ```
///
/// Alternatively, you can start the server and wait until it is stopped manually:
///
/// ```
/// use std::error::Error;
/// use brc20_prog::{Brc20ProgConfig, start};
///
/// pub async fn run_server() -> Result<(), Box<dyn Error>> {
///     let server_handle = start(Brc20ProgConfig::from_env()).await?;
///     // Wait until the server is stopped
///     server_handle.stopped().await;
///     Ok(())
/// }
/// ```
///
/// Only one instance of the server should be started in a single process. Configuration is shared so this
/// might cause issues if multiple instances are started in the same process.
pub async fn start(config: Brc20ProgConfig) -> Result<ServerHandle, Box<dyn Error>> {
    CONFIG.write_fn_unchecked(|value| {
        *value = config.clone();
    });

    validate_config(&config)?;

    match validate_bitcoin_rpc_status() {
        Ok(_) => info!("Bitcoin RPC status: OK"),
        Err(e) => {
            error!("Bitcoin RPC status: ERROR, Error: {}", e);
            if config.fail_on_bitcoin_rpc_error {
                return Err(format!("Bitcoin RPC status: ERROR\nError: {}", e).into());
            }
            warn!("Continuing without Bitcoin RPC status check");
        }
    }
    let engine = BRC20ProgEngine::new(DB::new(&Path::new(&config.db_path))?);
    info!("Latest block number: {}", engine.get_latest_block_height()?);
    start_rpc_server(engine, config).await
}
