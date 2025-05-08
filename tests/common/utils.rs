use std::net::TcpListener;

use brc20_prog::{start, Brc20ProgConfig};
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use jsonrpsee::server::ServerHandle;
use tempfile::TempDir;

fn get_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap(); // 0 = ask OS for free port
    let port = listener.local_addr().unwrap().port();
    port
}

pub async fn spawn_test_server() -> (ServerHandle, HttpClient) {
    let db_path = TempDir::new().unwrap();
    let server_address = format!("127.0.0.1:{}", get_free_port());
    let server = start(Brc20ProgConfig {
        db_path: db_path.path().to_str().unwrap().to_string(),
        brc20_prog_rpc_server_url: server_address.clone(),
        fail_on_bitcoin_rpc_error: false,
        ..Default::default()
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
