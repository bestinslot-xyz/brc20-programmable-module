use std::error::Error;
use std::path::Path;

mod brc20_controller;
pub mod evm;

mod db;
use db::DB;

mod server;
use evm::check_bitcoin_rpc_status;
use server::{start_rpc_server, ServerInstance};

lazy_static::lazy_static! {
    static ref BRC20_PROG_RPC_SERVER_ENABLE_AUTH: bool = std::env::var("BRC20_PROG_RPC_SERVER_ENABLE_AUTH").map(|x| x == "true").unwrap_or(false);
    static ref BRC20_PROG_RPC_SERVER_USER: Option<String> = std::env::var("BRC20_PROG_RPC_SERVER_USER").ok();
    static ref BRC20_PROG_RPC_SERVER_PASSWORD: Option<String> = std::env::var("BRC20_PROG_RPC_SERVER_PASSWORD").ok();
    static ref BRC20_PROG_RPC_SERVER_URL: String = std::env::var("BRC20_PROG_RPC_SERVER_URL").unwrap_or("127.0.0.1:18545".to_string());
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::INFO)
            .finish(),
    )?;

    let instance = ServerInstance::new(DB::new(&Path::new("target").join("db"))?);
    println!("--- Database ---");
    println!(
        "Latest block number: {}",
        instance.get_latest_block_height()
    );
    println!(
        "Genesis block hash: {}",
        instance
            .get_block_by_number(0, false)
            .map(|block| block.hash.0.to_string())
            .unwrap_or("None".to_string())
    );
    println!("");
    println!("--- Services ---");
    println!(
        "Bitcoin RPC status: {}",
        if check_bitcoin_rpc_status() {
            "OK"
        } else {
            "Error"
        }
    );
    println!("");
    println!("--- Server ---");
    println!(
        "Authentication enabled: {}",
        *BRC20_PROG_RPC_SERVER_ENABLE_AUTH
    );
    println!("Started JSON-RPC server on {}", *BRC20_PROG_RPC_SERVER_URL);
    let handle = start_rpc_server(
        BRC20_PROG_RPC_SERVER_URL.to_string(),
        instance,
        *BRC20_PROG_RPC_SERVER_ENABLE_AUTH,
        BRC20_PROG_RPC_SERVER_USER.as_ref(),
        BRC20_PROG_RPC_SERVER_PASSWORD.as_ref(),
    )
    .await?;

    handle.stopped().await;
    Ok(())
}
