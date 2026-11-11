//! HTTP basic auth, CORS and WebSocket behaviour of the listeners `start` and
//! `start_public_rpc_server` open, against real listeners on ephemeral ports.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use brc20_prog::{start, Brc20Prog, Brc20ProgApiClient, Brc20ProgConfig};
use jsonrpsee::core::client::{ClientT, Error as ClientError};
use jsonrpsee::core::params::BatchRequestBuilder;
use jsonrpsee::http_client::HeaderMap;
use jsonrpsee::rpc_params;
use jsonrpsee::server::ServerHandle;
use jsonrpsee::ws_client::WsClientBuilder;
use tempfile::TempDir;
use test_utils::{
    get_free_port, test_client, with_test_credentials, TEST_RPC_AUTH_HEADER, TEST_RPC_PASSWORD,
    TEST_RPC_USER,
};

struct Listener {
    handle: ServerHandle,
    address: String,
    _db_path: TempDir,
}

/// Starts `start` in single-listener mode with the test credentials.
async fn spawn_single_listener() -> Listener {
    let db_path = TempDir::new().unwrap();
    let address = format!("127.0.0.1:{}", get_free_port());
    let handle = start(with_test_credentials(Brc20ProgConfig {
        db_path: db_path.path().to_str().unwrap().to_string(),
        brc20_prog_rpc_server_url: address.clone(),
        brc20_prog_admin_rpc_server_url: None,
        fail_on_bitcoin_rpc_error: false,
        ..Default::default()
    }))
    .await
    .expect("Failed to start server");
    Listener {
        handle,
        address,
        _db_path: db_path,
    }
}

/// Opens the module in library mode and starts its public listener.
async fn spawn_library_listener() -> Listener {
    let db_path = TempDir::new().unwrap();
    let address = format!("127.0.0.1:{}", get_free_port());
    let prog = Brc20Prog::open(with_test_credentials(Brc20ProgConfig {
        db_path: db_path.path().to_str().unwrap().to_string(),
        brc20_prog_rpc_server_url: address.clone(),
        fail_on_bitcoin_rpc_error: false,
        ..Default::default()
    }))
    .expect("Failed to open brc20-prog");
    let handle = prog.start_public_rpc_server().await.unwrap();
    Listener {
        handle,
        address,
        _db_path: db_path,
    }
}

fn client_with_header(address: &str, header: &str) -> jsonrpsee::http_client::HttpClient {
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", header.parse().unwrap());
    jsonrpsee::http_client::HttpClientBuilder::default()
        .set_headers(headers)
        .build(format!("http://{}", address))
        .unwrap()
}

fn assert_unauthorized<T: std::fmt::Debug>(result: Result<T, ClientError>) {
    match result {
        Err(ClientError::Call(error)) => assert_eq!(error.code(), 401, "{error:?}"),
        other => panic!("expected a 401 call error, got {other:?}"),
    }
}

/// Sends a raw HTTP/1.1 request and returns the lowercased response head.
async fn raw_http_head(address: &str, request: String) -> String {
    let address = address.to_string();
    tokio::task::spawn_blocking(move || {
        let mut stream = TcpStream::connect(&address).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        stream.write_all(request.as_bytes()).unwrap();
        let mut response = Vec::new();
        let _ = stream.read_to_end(&mut response);
        let response = String::from_utf8_lossy(&response).to_lowercase();
        response
            .split("\r\n\r\n")
            .next()
            .unwrap_or_default()
            .to_string()
    })
    .await
    .unwrap()
}

async fn preflight_head(address: &str) -> String {
    raw_http_head(
        address,
        format!(
            "OPTIONS / HTTP/1.1\r\nHost: {address}\r\nOrigin: http://attacker.example\r\n\
             Access-Control-Request-Method: POST\r\n\
             Access-Control-Request-Headers: content-type, authorization\r\n\
             Connection: close\r\n\r\n"
        ),
    )
    .await
}

async fn post_with_origin_head(address: &str) -> String {
    let body = r#"{"jsonrpc":"2.0","id":1,"method":"eth_blockNumber","params":[]}"#;
    raw_http_head(
        address,
        format!(
            "POST / HTTP/1.1\r\nHost: {address}\r\nOrigin: http://attacker.example\r\n\
             Content-Type: application/json\r\nContent-Length: {}\r\n\
             Connection: close\r\n\r\n{body}",
            body.len()
        ),
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn test_single_listener_indexer_method_requires_auth() {
    let listener = spawn_single_listener().await;

    // No credentials
    let anonymous = test_client(&listener.address, false);
    assert_unauthorized(anonymous.brc20_mine(1, 0).await);
    assert_unauthorized(
        anonymous
            .brc20_deposit(
                "0014".to_string(),
                "test".to_string(),
                1u64.into(),
                0,
                [0; 32].into(),
                0,
                "inscription".to_string(),
            )
            .await,
    );
    assert_unauthorized(anonymous.brc20_commit_to_database().await);
    assert_unauthorized(anonymous.brc20_reorg(0).await);

    // Wrong password, and the right password for another user
    let wrong = client_with_header(&listener.address, "Basic aW5kZXhlcjp3cm9uZw==");
    assert_unauthorized(wrong.brc20_mine(1, 0).await);
    let other_user = format!(
        "Basic {}",
        BASE64_STANDARD.encode(format!("other-{TEST_RPC_USER}:{TEST_RPC_PASSWORD}"))
    );
    let wrong_user = client_with_header(&listener.address, &other_user);
    assert_unauthorized(wrong_user.brc20_mine(1, 0).await);

    // Nothing was mined by the rejected calls
    assert_eq!(anonymous.eth_block_number().await.unwrap(), "0x0");

    // Correct credentials
    let authorized = test_client(&listener.address, true);
    authorized.brc20_mine(2, 0).await.unwrap();
    assert_eq!(authorized.eth_block_number().await.unwrap(), "0x1");

    listener.handle.stop().unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_single_listener_eth_methods_need_no_auth() {
    let listener = spawn_single_listener().await;
    let anonymous = test_client(&listener.address, false);

    assert!(anonymous.eth_block_number().await.is_ok());
    assert!(anonymous.eth_chain_id().await.is_ok());

    listener.handle.stop().unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_single_listener_batch_rejects_only_indexer_entries() {
    let listener = spawn_single_listener().await;

    let batch = || {
        let mut batch = BatchRequestBuilder::new();
        batch.insert("brc20_mine", rpc_params![1, 0]).unwrap();
        batch.insert("eth_blockNumber", rpc_params![]).unwrap();
        batch
            .insert("brc20_commitToDatabase", rpc_params![])
            .unwrap();
        batch.insert("eth_chainId", rpc_params![]).unwrap();
        batch
    };

    // Without credentials the indexer entries fail with 401, the public entries succeed
    let anonymous = test_client(&listener.address, false);
    let responses: Vec<_> = anonymous
        .batch_request::<serde_json::Value>(batch())
        .await
        .unwrap()
        .into_iter()
        .collect();
    assert_eq!(responses.len(), 4);
    assert_eq!(responses[0].as_ref().unwrap_err().code(), 401);
    assert!(responses[1].is_ok());
    assert_eq!(responses[2].as_ref().unwrap_err().code(), 401);
    assert!(responses[3].is_ok());

    // With credentials every entry succeeds
    let authorized = test_client(&listener.address, true);
    let responses: Vec<_> = authorized
        .batch_request::<serde_json::Value>(batch())
        .await
        .unwrap()
        .into_iter()
        .collect();
    assert!(
        responses.iter().all(|response| response.is_ok()),
        "{responses:?}"
    );

    listener.handle.stop().unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_single_listener_sends_no_cors_headers() {
    let listener = spawn_single_listener().await;

    let preflight = preflight_head(&listener.address).await;
    assert!(
        !preflight.contains("access-control-allow-origin"),
        "{preflight}"
    );
    let post = post_with_origin_head(&listener.address).await;
    assert!(post.starts_with("http/1.1 200"), "{post}");
    assert!(!post.contains("access-control-allow-origin"), "{post}");

    listener.handle.stop().unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_single_listener_refuses_websocket() {
    let listener = spawn_single_listener().await;
    let url = format!("ws://{}", listener.address);

    assert!(WsClientBuilder::default().build(&url).await.is_err());

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", TEST_RPC_AUTH_HEADER.parse().unwrap());
    assert!(WsClientBuilder::default()
        .set_headers(headers)
        .build(&url)
        .await
        .is_err());

    listener.handle.stop().unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_split_listeners_cors() {
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
    .unwrap();

    let admin = preflight_head(&admin_address).await;
    assert!(!admin.contains("access-control-allow-origin"), "{admin}");

    let public = preflight_head(&public_address).await;
    assert!(
        public.contains("access-control-allow-origin: *"),
        "{public}"
    );

    handle.stop().unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_library_listener_rejects_indexer_methods_with_credentials() {
    let listener = spawn_library_listener().await;

    let authorized = test_client(&listener.address, true);
    assert_unauthorized(authorized.brc20_mine(1, 0).await);
    assert_unauthorized(authorized.brc20_commit_to_database().await);
    assert!(authorized.eth_block_number().await.is_ok());

    let mut batch = BatchRequestBuilder::new();
    batch.insert("brc20_mine", rpc_params![1, 0]).unwrap();
    batch.insert("eth_blockNumber", rpc_params![]).unwrap();
    let responses: Vec<_> = authorized
        .batch_request::<serde_json::Value>(batch)
        .await
        .unwrap()
        .into_iter()
        .collect();
    assert_eq!(responses[0].as_ref().unwrap_err().code(), 401);
    assert!(responses[1].is_ok());

    listener.handle.stop().unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_library_listener_rejects_indexer_methods_over_websocket() {
    let listener = spawn_library_listener().await;

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", TEST_RPC_AUTH_HEADER.parse().unwrap());
    let ws = WsClientBuilder::default()
        .set_headers(headers)
        .build(format!("ws://{}", listener.address))
        .await
        .unwrap();
    assert_unauthorized(ws.brc20_mine(1, 0).await);
    assert!(ws.eth_block_number().await.is_ok());

    let anonymous = WsClientBuilder::default()
        .build(format!("ws://{}", listener.address))
        .await
        .unwrap();
    assert_unauthorized(anonymous.brc20_mine(1, 0).await);

    listener.handle.stop().unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_start_refuses_without_strong_credentials() {
    let db_path = TempDir::new().unwrap();
    let address = format!("127.0.0.1:{}", get_free_port());
    let base = || Brc20ProgConfig {
        db_path: db_path.path().to_str().unwrap().to_string(),
        brc20_prog_rpc_server_url: address.clone(),
        brc20_prog_admin_rpc_server_url: None,
        fail_on_bitcoin_rpc_error: false,
        ..Default::default()
    };

    let missing = Brc20ProgConfig {
        brc20_prog_rpc_server_enable_auth: true,
        brc20_prog_rpc_server_user: None,
        brc20_prog_rpc_server_password: None,
        ..base()
    };
    assert!(start(missing).await.is_err());

    let weak = Brc20ProgConfig {
        brc20_prog_rpc_server_password: Some("password".to_string()),
        ..with_test_credentials(base())
    };
    assert!(start(weak).await.is_err());

    let disabled = Brc20ProgConfig {
        brc20_prog_rpc_server_enable_auth: false,
        ..with_test_credentials(base())
    };
    let error = start(disabled).await.unwrap_err().to_string();
    assert!(error.contains("ENABLE_AUTH=false"), "{error}");

    // Nothing is listening after the refusals
    assert!(TcpStream::connect(&address).is_err());
}
