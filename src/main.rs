use std::error::Error;
use std::path::Path;

mod brc20_controller;
mod config;
mod db;
mod evm;
mod server;

use config::{validate_config, BRC20_PROG_CONFIG};
use db::DB;
use server::{start_rpc_server, BRC20ProgEngine};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv()?;

    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::INFO)
            .finish(),
    )?;

    println!("BRC20 Prog v{}", BRC20_PROG_CONFIG.pkg_version);
    validate_config()?;

    let engine = BRC20ProgEngine::new(DB::new(&Path::new(&*(BRC20_PROG_CONFIG).db_path))?);
    println!("--- Database ---");
    println!("Latest block number: {}", engine.get_latest_block_height()?);
    println!(
        "Genesis block hash: {}",
        engine
            .get_block_by_number(0, false)?
            .map(|block| block.hash.bytes.to_string())
            .unwrap_or("None".to_string())
    );
    println!("");
    println!("--- Server ---");
    println!(
        "Authentication enabled: {}",
        (*BRC20_PROG_CONFIG).brc20_prog_rpc_server_enable_auth
    );
    println!(
        "Started JSON-RPC server on {}",
        (*BRC20_PROG_CONFIG).brc20_prog_rpc_server_url
    );
    let handle = start_rpc_server(
        engine,
        (*BRC20_PROG_CONFIG).brc20_prog_rpc_server_url.to_string(),
        (*BRC20_PROG_CONFIG).brc20_prog_rpc_server_enable_auth,
        (*BRC20_PROG_CONFIG).brc20_prog_rpc_server_user.as_ref(),
        (*BRC20_PROG_CONFIG).brc20_prog_rpc_server_password.as_ref(),
    )
    .await?;

    handle.stopped().await;
    Ok(())
}
