use std::{error::Error, path::Path};

use db::DB;
use server::{start_rpc_server, ServerInstance};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1:18545";
    let instance = ServerInstance::new(DB::new(&Path::new("target").join("db")).unwrap());
    println!("Started JSON-RPC server on {}", addr);
    let handle = start_rpc_server(addr, instance).await?;
    handle.stopped().await;
    Ok(())
}
