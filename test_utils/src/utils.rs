use std::error::Error;
use std::io::{Read, Write};
use std::net::TcpListener;

use brc20_prog::types::{EthBytes, EthCall};
use brc20_prog::{start, Brc20ProgApiClient, Brc20ProgConfig};
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use jsonrpsee::server::ServerHandle;
use rust_embed::Embed;
use tempfile::TempDir;
use tokio::runtime::Runtime;

#[derive(Embed)]
#[folder = "data"]
struct TxAssets;

fn get_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap(); // 0 = ask OS for free port
    let port = listener.local_addr().unwrap().port();
    port
}

pub async fn spawn_test_server(config: Brc20ProgConfig) -> (ServerHandle, HttpClient) {
    let db_path = TempDir::new().unwrap();
    let server_address = format!("127.0.0.1:{}", get_free_port());
    let server = start(Brc20ProgConfig {
        db_path: db_path.path().to_str().unwrap().to_string(),
        brc20_prog_rpc_server_url: server_address.clone(),
        fail_on_bitcoin_rpc_error: false,
        fail_on_brc20_balance_server_error: false,
        ..config
    })
    .await
    .expect("Failed to start server");

    (
        server,
        HttpClientBuilder::default()
            .build(format!("http://{}", server_address))
            .expect("Failed to create client"),
    )
}

pub fn spawn_balance_server() {
    std::thread::spawn(|| {
        let listener = TcpListener::bind("127.0.0.1:18546").unwrap();
        loop {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0; 1024];
            let _ = stream.read(&mut buf).unwrap();
            let response = b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\n100";
            let _ = stream.write(response).unwrap();
        }
    });
}

pub fn is_in_ci() -> bool {
    // Check if the environment variable "CI" is set to "true"
    std::env::var("CI").map_or(false, |val| val == "true")
}

pub fn load_file_as_string(filename: &str) -> Result<String, Box<dyn Error>> {
    Ok(String::from_utf8(
        TxAssets::get(filename)
            .expect("Failed to load file")
            .data
            .to_vec(),
    )?)
}

pub fn load_file_as_eth_bytes(filename: &str) -> Result<EthBytes, Box<dyn Error>> {
    Ok(EthBytes::new(load_file_as_string(filename)?))
}

pub fn print_gas_per_call(rt: &Runtime, client: &HttpClient, eth_call: EthCall) -> u64 {
    let gas_per_call = u64::from_str_radix(
        rt.block_on(async {
            client
                .eth_estimate_gas(eth_call.clone(), None)
                .await
                .unwrap()
        })
        .trim_start_matches("0x"),
        16,
    )
    .unwrap();

    println!("Gas per call: {}", gas_per_call);
    println!(
        "Calldata bytes: {}",
        eth_call.data.unwrap().to_string().len()
    );
    println!("Required bytes: {}", gas_per_call / 12000);
    println!("");

    gas_per_call
}
