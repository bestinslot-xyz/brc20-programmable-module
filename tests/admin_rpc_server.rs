use std::net::TcpListener;

use alloy::primitives::U256;
use alloy_consensus::TxLegacy;
use alloy_network::{EthereumWallet, NetworkTransactionBuilder, TransactionBuilder};
use alloy_rpc_types_eth::TransactionRequest;
use alloy_signer_local::PrivateKeySigner;
use brc20_prog::types::RawBytes;
use brc20_prog::{start, Brc20ProgApiClient, Brc20ProgConfig};
use jsonrpsee::core::client::Error as ClientError;
use jsonrpsee::http_client::{HeaderMap, HttpClient, HttpClientBuilder};
use jsonrpsee::server::ServerHandle;
use revm::primitives::Address;
use tempfile::TempDir;
use test_utils::get_free_port;

// "user:password" in base64
const BASIC_AUTH_HEADER: &str = "Basic dXNlcjpwYXNzd29yZA==";

struct SplitServer {
    handle: ServerHandle,
    public_address: String,
    admin_address: String,
    _db_path: TempDir,
}

async fn spawn_split_server(enable_auth: bool) -> SplitServer {
    let db_path = TempDir::new().unwrap();
    let public_address = format!("127.0.0.1:{}", get_free_port());
    let admin_address = format!("127.0.0.1:{}", get_free_port());
    let handle = start(Brc20ProgConfig {
        db_path: db_path.path().to_str().unwrap().to_string(),
        brc20_prog_rpc_server_url: public_address.clone(),
        brc20_prog_admin_rpc_server_url: Some(admin_address.clone()),
        brc20_prog_rpc_server_enable_auth: enable_auth,
        brc20_prog_rpc_server_user: Some("user".to_string()),
        brc20_prog_rpc_server_password: Some("password".to_string()),
        fail_on_bitcoin_rpc_error: false,
        ..Default::default()
    })
    .await
    .expect("Failed to start server");

    SplitServer {
        handle,
        public_address,
        admin_address,
        _db_path: db_path,
    }
}

fn client(address: &str, with_credentials: bool) -> HttpClient {
    let mut headers = HeaderMap::new();
    if with_credentials {
        headers.insert("Authorization", BASIC_AUTH_HEADER.parse().unwrap());
    }
    HttpClientBuilder::default()
        .set_headers(headers)
        .build(format!("http://{}", address))
        .expect("Failed to create client")
}

#[tokio::test]
async fn test_public_server_rejects_indexer_methods_with_credentials() {
    let server = spawn_split_server(true).await;
    let public = client(&server.public_address, true);

    assert!(public.brc20_mine(1, 0).await.is_err());
    assert!(public.brc20_commit_to_database().await.is_err());
    assert!(public.eth_block_number().await.is_ok());

    server.handle.stop().unwrap();
}

#[tokio::test]
async fn test_public_server_rejects_indexer_methods_without_auth() {
    let server = spawn_split_server(false).await;
    let public = client(&server.public_address, false);

    assert!(public.brc20_mine(1, 0).await.is_err());
    assert!(public.eth_block_number().await.is_ok());

    server.handle.stop().unwrap();
}

#[tokio::test]
async fn test_admin_server_serves_indexer_methods() {
    let server = spawn_split_server(true).await;
    let admin = client(&server.admin_address, true);

    admin.brc20_mine(1, 0).await.unwrap();
    assert_eq!(admin.eth_block_number().await.unwrap(), "0x0");

    // The public server sees the block mined through the admin server
    let public = client(&server.public_address, false);
    assert_eq!(public.eth_block_number().await.unwrap(), "0x0");

    server.handle.stop().unwrap();
}

#[tokio::test]
async fn test_admin_server_requires_credentials_when_auth_enabled() {
    let server = spawn_split_server(true).await;
    let admin = client(&server.admin_address, false);

    assert!(admin.brc20_mine(1, 0).await.is_err());

    server.handle.stop().unwrap();
}

#[tokio::test]
async fn test_only_public_server_rate_limits_send_raw() {
    let server = spawn_split_server(false).await;
    let public = client(&server.public_address, false);
    let admin = client(&server.admin_address, false);

    let chain_id = public.eth_chain_id().await.unwrap();
    let chain_id = u64::from_str_radix(chain_id.trim_start_matches("0x"), 16).unwrap();
    let wallet = EthereumWallet::new(PrivateKeySigner::random());
    let tx_builder: TransactionRequest = TxLegacy::default().into();
    let signed_tx = tx_builder
        .with_chain_id(chain_id)
        .with_nonce(0)
        .with_gas_price(0)
        .with_gas_limit(0)
        .with_to(Address::ZERO)
        .with_value(U256::ZERO)
        .build(&wallet)
        .await
        .unwrap()
        .into_signed();
    let mut encoded = Vec::new();
    signed_tx.network_encode(&mut encoded);
    let tx = RawBytes::new(format!("0x{}", hex::encode(&encoded)));

    for _ in 0..30 {
        public.eth_send_raw_transaction(tx.clone()).await.unwrap();
    }
    match public
        .eth_send_raw_transaction(tx.clone())
        .await
        .unwrap_err()
    {
        ClientError::Call(error) => assert_eq!(error.code(), -32005),
        other => panic!("expected a call error, got {other:?}"),
    }

    for _ in 0..31 {
        admin.eth_send_raw_transaction(tx.clone()).await.unwrap();
    }

    server.handle.stop().unwrap();
}

#[tokio::test]
async fn test_stop_releases_both_ports() {
    let server = spawn_split_server(false).await;

    server.handle.stop().unwrap();
    server.handle.clone().stopped().await;

    assert!(TcpListener::bind(&server.public_address).is_ok());
    assert!(TcpListener::bind(&server.admin_address).is_ok());
}
