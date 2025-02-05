use std::error::Error;

use db::DB;
use server::{start_http_server, start_rpc_server, ServerInstance};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().collect::<Vec<String>>();
    let addr = "127.0.0.1:18545";
    let instance = ServerInstance::new(DB::new().unwrap());
    if args.len() > 1 && args[1] == "rpc" {
        println!("Started JSON-RPC server on {}", addr);
        let handle = start_rpc_server(addr, instance).await?;
        handle.stopped().await;
    } else {
        println!("Started HTTP server on {}", addr);
        start_http_server(addr, instance);
    }
    Ok(())
}
