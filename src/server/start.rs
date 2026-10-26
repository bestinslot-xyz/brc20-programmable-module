use std::error::Error;

use jsonrpsee::server::ServerHandle;

use crate::global::Brc20ProgConfig;
use crate::server::rpc_server::{start_rpc_server, Brc20Prog};

/// Starts the BRC20 programmable module server.
///
/// This function initializes the logging, validates the configuration, initializes the database, and starts the JSON-RPC server.
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
/// If `brc20_prog_admin_rpc_server_url` is set, these methods are served only on the admin listener,
/// and the public listener rejects them for every caller. The returned handle controls both listeners.
///
/// # Errors
///
/// This function will return an error if:
/// * The logging initialization fails.
/// * The configuration validation fails.
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
    start_rpc_server(Brc20Prog::open(config)?).await
}
