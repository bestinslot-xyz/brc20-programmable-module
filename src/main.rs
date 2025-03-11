use std::{error::Error, path::Path};

use db::DB;
use server::{start_rpc_server, ServerInstance};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::INFO)
            .finish(),
    )?;

    let addr = "127.0.0.1:18545";
    let instance = ServerInstance::new(DB::new(&Path::new("target").join("db")).unwrap());
    println!(
        "Latest block number: {}",
        instance.get_latest_block_height()
    );
    println!(
        "Genesis block hash: {}",
        instance.get_block_by_number(0).map(|block| block.hash.0.to_string()).unwrap_or("None".to_string())
    );
    println!("Started JSON-RPC server on {}", addr);
    let handle = start_rpc_server(addr, instance).await?;
    handle.stopped().await;
    Ok(())
}
