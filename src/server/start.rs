use std::error::Error;

use jsonrpsee::server::ServerHandle;

use crate::global::{validate_rpc_server_credentials, Brc20ProgConfig};
use crate::server::rpc_server::{start_rpc_server, Brc20Prog};

/// Starts the BRC20 programmable module in server mode.
///
/// This function validates the configuration, opens the database, and starts the JSON-RPC server.
///
/// Server can be configured by passing a `Brc20ProgConfig` instance.
///
/// # Server mode and library mode
///
/// In server mode the indexer drives the module over JSON-RPC, so every listener that serves the
/// following indexer methods requires HTTP basic auth with `brc20_prog_rpc_server_user` and
/// `brc20_prog_rpc_server_password`. Every other method is accessible without authentication.
/// * brc20_mine
/// * brc20_deploy
/// * brc20_call
/// * brc20_transact
/// * brc20_deposit
/// * brc20_withdraw
/// * brc20_initialise
/// * brc20_finaliseBlock
/// * brc20_reorg
/// * brc20_commitToDatabase
/// * brc20_clearCaches
/// * debug_getBlockTraceString
/// * debug_getBlockTraceHash
///
/// A listener that serves the indexer methods sends no CORS headers and serves HTTP only, no
/// WebSocket.
///
/// If `brc20_prog_admin_rpc_server_url` is set, these methods are served only on the admin listener,
/// and the public listener does not register them, answering calls to them with a 401 error for
/// every caller. The public listener allows requests from any origin. The returned handle
/// controls both listeners.
///
/// An indexer written in Rust that wants no network access to the indexer methods uses library
/// mode instead: [`Brc20Prog::open`] opens the module in process, the indexer calls the indexer
/// methods directly, and [`Brc20Prog::start_public_rpc_server`] optionally serves the public
/// methods to other clients without registering the indexer methods. Library mode
/// needs no credentials.
///
/// # Errors
///
/// This function will return an error, before opening the database or binding any port, if:
/// * The username or password is missing, empty, or weak: shorter than 32 characters, a known
///   sample value such as `password`, equal to the username, or a single repeated character.
///   Generate one with `openssl rand -hex 32`.
/// * `brc20_prog_rpc_server_enable_auth` is false. Authentication is mandatory for the indexer
///   methods in server mode.
///
/// It also returns an error if:
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
///     // Reads BRC20_PROG_RPC_SERVER_USER and BRC20_PROG_RPC_SERVER_PASSWORD among others
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
    validate_rpc_server_credentials(&config)?;
    start_rpc_server(Brc20Prog::open(config)?).await
}
