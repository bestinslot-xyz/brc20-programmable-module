use std::net::TcpListener;

use brc20_prog::{start, Brc20ProgApiClient, Brc20ProgConfig};
use jsonrpsee::server::ServerHandle;
use tempfile::TempDir;
use test_utils::{get_free_port, test_client as client, with_test_credentials};

struct SplitServer {
    handle: ServerHandle,
    public_address: String,
    admin_address: String,
    _db_path: TempDir,
}

async fn spawn_split_server() -> SplitServer {
    let db_path = TempDir::new().unwrap();
    let public_address = format!("127.0.0.1:{}", get_free_port());
    let admin_address = format!("127.0.0.1:{}", get_free_port());
    let handle = start(with_test_credentials(Brc20ProgConfig {
        db_path: db_path.path().to_str().unwrap().to_string(),
        brc20_prog_rpc_server_url: public_address.clone(),
        brc20_prog_admin_rpc_server_url: Some(admin_address.clone()),
        fail_on_bitcoin_rpc_error: false,
        ..Default::default()
    }))
    .await
    .expect("Failed to start server");

    SplitServer {
        handle,
        public_address,
        admin_address,
        _db_path: db_path,
    }
}

#[tokio::test]
async fn test_public_server_rejects_indexer_methods_with_credentials() {
    let server = spawn_split_server().await;
    let public = client(&server.public_address, true);

    assert!(public.brc20_mine(1, 0).await.is_err());
    assert!(public.brc20_commit_to_database().await.is_err());
    assert!(public.eth_block_number().await.is_ok());

    server.handle.stop().unwrap();
}

#[tokio::test]
async fn test_public_server_rejects_indexer_methods_without_auth() {
    let server = spawn_split_server().await;
    let public = client(&server.public_address, false);

    assert!(public.brc20_mine(1, 0).await.is_err());
    assert!(public.eth_block_number().await.is_ok());

    server.handle.stop().unwrap();
}

#[tokio::test]
async fn test_admin_server_serves_indexer_methods() {
    let server = spawn_split_server().await;
    let admin = client(&server.admin_address, true);

    admin.brc20_mine(1, 0).await.unwrap();
    assert_eq!(admin.eth_block_number().await.unwrap(), "0x0");

    // The public server sees the block mined through the admin server
    let public = client(&server.public_address, false);
    assert_eq!(public.eth_block_number().await.unwrap(), "0x0");

    server.handle.stop().unwrap();
}

#[tokio::test]
async fn test_admin_server_requires_credentials() {
    let server = spawn_split_server().await;
    let admin = client(&server.admin_address, false);

    assert!(admin.brc20_mine(1, 0).await.is_err());
    assert!(admin.eth_block_number().await.is_ok());

    server.handle.stop().unwrap();
}

#[tokio::test]
async fn test_stop_releases_both_ports() {
    let server = spawn_split_server().await;

    server.handle.stop().unwrap();
    server.handle.clone().stopped().await;

    assert!(TcpListener::bind(&server.public_address).is_ok());
    assert!(TcpListener::bind(&server.admin_address).is_ok());
}
