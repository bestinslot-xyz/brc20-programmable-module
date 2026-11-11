use std::error::Error;
use std::net::TcpListener;

use brc20_prog::types::{EthCall, RawBytes};
use brc20_prog::{start, Brc20ProgApiClient, Brc20ProgConfig};
use jsonrpsee::http_client::{HeaderMap, HttpClient, HttpClientBuilder};
use jsonrpsee::server::ServerHandle;
use rust_embed::Embed;
use tempfile::TempDir;
use tokio::runtime::Runtime;

#[derive(Embed)]
#[folder = "data"]
struct TxAssets;

pub fn get_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap(); // 0 = ask OS for free port
    let port = listener.local_addr().unwrap().port();
    port
}

/// Username that test servers require for the indexer methods.
pub const TEST_RPC_USER: &str = "indexer";
/// Password that test servers require for the indexer methods. Test-only value.
pub const TEST_RPC_PASSWORD: &str =
    "6fb774103c949efaa166f67f0647b32fa0b7fb2ee9d9c222c395df6a89f959fd";
/// `Authorization` header value for [`TEST_RPC_USER`] and [`TEST_RPC_PASSWORD`].
pub const TEST_RPC_AUTH_HEADER: &str = "Basic aW5kZXhlcjo2ZmI3NzQxMDNjOTQ5ZWZhYTE2NmY2N2YwNjQ3YjMyZmEwYjdmYjJlZTlkOWMyMjJjMzk1ZGY2YTg5Zjk1OWZk";

/// Returns `config` with the test credentials set.
pub fn with_test_credentials(config: Brc20ProgConfig) -> Brc20ProgConfig {
    Brc20ProgConfig {
        brc20_prog_rpc_server_enable_auth: true,
        brc20_prog_rpc_server_user: Some(TEST_RPC_USER.to_string()),
        brc20_prog_rpc_server_password: Some(TEST_RPC_PASSWORD.to_string()),
        ..config
    }
}

/// Builds a client for `address`, sending the test credentials when `authorized` is true.
pub fn test_client(address: &str, authorized: bool) -> HttpClient {
    let mut headers = HeaderMap::new();
    if authorized {
        headers.insert("Authorization", TEST_RPC_AUTH_HEADER.parse().unwrap());
    }
    HttpClientBuilder::default()
        .set_headers(headers)
        .build(format!("http://{}", address))
        .expect("Failed to create client")
}

/// Starts a single-listener server with the test credentials, and returns it with a client that
/// sends them.
pub async fn spawn_test_server(config: Brc20ProgConfig) -> (ServerHandle, HttpClient) {
    let db_path = TempDir::new().unwrap();
    let server_address = format!("127.0.0.1:{}", get_free_port());
    let server = start(with_test_credentials(Brc20ProgConfig {
        db_path: db_path.path().to_str().unwrap().to_string(),
        brc20_prog_rpc_server_url: server_address.clone(),
        fail_on_bitcoin_rpc_error: false,
        evm_record_traces: true,
        ..config
    }))
    .await
    .expect("Failed to start server");

    (server, test_client(&server_address, true))
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

pub fn load_file_as_eth_bytes(filename: &str) -> Result<RawBytes, Box<dyn Error>> {
    Ok(RawBytes::new(load_file_as_string(filename)?))
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
