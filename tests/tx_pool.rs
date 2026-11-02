use std::str::FromStr;

use alloy::eips::eip2718::Encodable2718;
use alloy::primitives::{keccak256, Bytes, B256, U256};
use alloy_consensus::TxLegacy;
use alloy_network::{EthereumWallet, NetworkTransactionBuilder, TransactionBuilder};
use alloy_rpc_types_eth::TransactionRequest;
use alloy_signer_local::PrivateKeySigner;
use brc20_prog::types::{Base64Bytes, RawBytes};
use brc20_prog::{start, Brc20ProgApiClient, Brc20ProgConfig};
use jsonrpsee::core::client::Error as ClientError;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use revm::primitives::Address;
use tempfile::TempDir;
use test_utils::{get_free_port, load_file_as_string, spawn_test_server};

const INVALID_PARAMS: i32 = -32602;
const CALLDATA_LIMIT: usize = 1024 * 1024;

async fn chain_id(client: &HttpClient) -> u64 {
    let chain_id = client.eth_chain_id().await.unwrap();
    u64::from_str_radix(chain_id.trim_start_matches("0x"), 16).unwrap()
}

/// Signs a legacy transaction and returns its network encoding.
async fn sign_legacy(
    wallet: &EthereumWallet,
    chain_id: u64,
    nonce: u64,
    value: U256,
    input: Bytes,
) -> Vec<u8> {
    let tx_builder: TransactionRequest = TxLegacy::default().into();
    let tx_builder = tx_builder
        .with_chain_id(chain_id)
        .with_nonce(nonce)
        .with_gas_price(0)
        .with_gas_limit(0)
        .with_to(Address::ZERO)
        .with_value(value)
        .with_input(input);
    let signed_tx = tx_builder.build(wallet).await.unwrap().into_signed();
    let mut encoded = Vec::new();
    signed_tx.network_encode(&mut encoded);
    encoded
}

fn raw(bytes: &[u8]) -> RawBytes {
    RawBytes::new(format!("0x{}", hex::encode(bytes)))
}

fn error_code(error: ClientError) -> i32 {
    match error {
        ClientError::Call(error) => error.code(),
        other => panic!("expected a call error, got {other:?}"),
    }
}

#[tokio::test]
async fn send_raw_returns_keccak_and_get_returns_raw() {
    let (server, client) = spawn_test_server(Default::default()).await;
    let wallet = EthereumWallet::new(PrivateKeySigner::random());
    let tx = sign_legacy(
        &wallet,
        chain_id(&client).await,
        0,
        U256::ZERO,
        Bytes::new(),
    )
    .await;

    let hash = client.eth_send_raw_transaction(raw(&tx)).await.unwrap();
    assert_eq!(hash.bytes, keccak256(&tx));

    let fetched = client
        .eth_get_raw_transaction_by_hash(hash)
        .await
        .unwrap()
        .expect("pooled tx");
    assert_eq!(fetched.to_string(), format!("0x{}", hex::encode(&tx)));

    let missing = client
        .eth_get_raw_transaction_by_hash(B256::repeat_byte(9).into())
        .await
        .unwrap();
    assert!(missing.is_none());

    server.stop().unwrap();
}

#[tokio::test]
async fn send_raw_rejects_wrong_chain_id() {
    let (server, client) = spawn_test_server(Default::default()).await;
    let wallet = EthereumWallet::new(PrivateKeySigner::random());
    let tx = sign_legacy(
        &wallet,
        chain_id(&client).await + 1,
        0,
        U256::ZERO,
        Bytes::new(),
    )
    .await;

    let error = client.eth_send_raw_transaction(raw(&tx)).await.unwrap_err();
    assert_eq!(error_code(error), INVALID_PARAMS);

    server.stop().unwrap();
}

#[tokio::test]
async fn send_raw_rejects_nonce_below_account_nonce() {
    let (server, client) = spawn_test_server(Default::default()).await;
    let chain_id = chain_id(&client).await;
    let wallet = EthereumWallet::new(PrivateKeySigner::random());
    let deploy_data =
        Bytes::from_str(&load_file_as_string("brc20_prog_helper_deploy_tx_data").unwrap()).unwrap();

    // Execute nonce 0 so the account nonce becomes 1
    let tx0 = sign_legacy(&wallet, chain_id, 0, U256::ZERO, deploy_data.clone()).await;
    let receipts = client
        .brc20_transact(
            None,
            Base64Bytes::from_bytes(tx0.clone().into()).unwrap().into(),
            42,
            [0u8; 32].into(),
            0,
            "deploy_inscription".to_string(),
            tx0.len() as u64,
            [1; 32].into(),
        )
        .await
        .unwrap();
    assert_eq!(receipts.len(), 1);

    let stale = sign_legacy(&wallet, chain_id, 0, U256::ZERO, Bytes::new()).await;
    let error = client
        .eth_send_raw_transaction(raw(&stale))
        .await
        .unwrap_err();
    assert_eq!(error_code(error), INVALID_PARAMS);

    server.stop().unwrap();
}

#[tokio::test]
async fn send_raw_rejects_nonce_beyond_future_window() {
    let (server, client) = spawn_test_server(Default::default()).await;
    let chain_id = chain_id(&client).await;
    let wallet = EthereumWallet::new(PrivateKeySigner::random());

    let in_window = sign_legacy(&wallet, chain_id, 9, U256::ZERO, Bytes::new()).await;
    client
        .eth_send_raw_transaction(raw(&in_window))
        .await
        .unwrap();

    let beyond = sign_legacy(&wallet, chain_id, 10, U256::ZERO, Bytes::new()).await;
    let error = client
        .eth_send_raw_transaction(raw(&beyond))
        .await
        .unwrap_err();
    assert_eq!(error_code(error), INVALID_PARAMS);

    server.stop().unwrap();
}

#[tokio::test]
async fn send_raw_rejects_oversize() {
    let (server, client) = spawn_test_server(Default::default()).await;

    let oversize = vec![0u8; CALLDATA_LIMIT + 1];
    let error = client
        .eth_send_raw_transaction(raw(&oversize))
        .await
        .unwrap_err();
    assert_eq!(error_code(error), INVALID_PARAMS);

    server.stop().unwrap();
}

#[tokio::test]
async fn send_raw_rejects_type2_tx() {
    let (server, client) = spawn_test_server(Default::default()).await;
    let wallet = EthereumWallet::new(PrivateKeySigner::random());

    let tx_builder = TransactionRequest::default()
        .with_chain_id(chain_id(&client).await)
        .with_nonce(0)
        .with_max_fee_per_gas(1)
        .with_max_priority_fee_per_gas(0)
        .with_gas_limit(21_000)
        .with_to(Address::ZERO)
        .with_value(U256::ZERO);
    // Wallets submit typed transactions in their EIP-2718 encoding
    let tx = tx_builder.build(&wallet).await.unwrap().encoded_2718();
    assert_eq!(tx[0], 0x02);

    let error = client.eth_send_raw_transaction(raw(&tx)).await.unwrap_err();
    assert_eq!(error_code(error), INVALID_PARAMS);

    server.stop().unwrap();
}

#[tokio::test]
async fn send_raw_rejects_nonzero_value() {
    let (server, client) = spawn_test_server(Default::default()).await;
    let wallet = EthereumWallet::new(PrivateKeySigner::random());
    let tx = sign_legacy(
        &wallet,
        chain_id(&client).await,
        0,
        U256::from(1),
        Bytes::new(),
    )
    .await;

    let error = client.eth_send_raw_transaction(raw(&tx)).await.unwrap_err();
    let message = match &error {
        ClientError::Call(error) => error.message().to_string(),
        other => panic!("expected a call error, got {other:?}"),
    };
    assert_eq!(error_code(error), INVALID_PARAMS);
    assert!(message.contains("value"), "message: {message}");

    server.stop().unwrap();
}

#[tokio::test]
async fn unsent_list_contains_pooled_tx() {
    let (server, client) = spawn_test_server(Default::default()).await;
    let signer = PrivateKeySigner::random();
    let from = signer.address();
    let wallet = EthereumWallet::new(signer);
    let chain_id = chain_id(&client).await;

    let tx0 = sign_legacy(&wallet, chain_id, 0, U256::ZERO, Bytes::new()).await;
    let tx1 = sign_legacy(&wallet, chain_id, 1, U256::ZERO, Bytes::new()).await;
    let hash0 = client.eth_send_raw_transaction(raw(&tx0)).await.unwrap();
    let hash1 = client.eth_send_raw_transaction(raw(&tx1)).await.unwrap();

    let unsent = client.brc20_unsent_raw_transactions().await.unwrap();
    assert_eq!(unsent.len(), 2);
    assert_eq!(unsent[0].hash, hash0);
    assert_eq!(unsent[1].hash, hash1);
    assert_eq!(
        unsent[0].raw.to_string(),
        format!("0x{}", hex::encode(&tx0))
    );
    assert_eq!(unsent[0].from.address, from);
    assert_eq!(Into::<u64>::into(unsent[1].nonce.clone()), 1);
    assert!(unsent[0].received_at_secs < 60);

    let json = serde_json::to_value(&unsent[0]).unwrap();
    assert!(json.get("receivedAtSecs").is_some(), "json: {json}");

    server.stop().unwrap();
}

#[tokio::test]
async fn pending_nonce_follows_pooled_txs() {
    let (server, client) = spawn_test_server(Default::default()).await;
    let signer = PrivateKeySigner::random();
    let from = signer.address();
    let wallet = EthereumWallet::new(signer);
    let chain_id = chain_id(&client).await;

    let tx0 = sign_legacy(&wallet, chain_id, 0, U256::ZERO, Bytes::new()).await;
    let tx1 = sign_legacy(&wallet, chain_id, 1, U256::ZERO, Bytes::new()).await;
    let tx1_resigned = sign_legacy(&wallet, chain_id, 1, U256::ZERO, Bytes::from(vec![1u8])).await;
    assert_ne!(tx1, tx1_resigned);
    for tx in [&tx0, &tx1, &tx1_resigned] {
        client.eth_send_raw_transaction(raw(tx)).await.unwrap();
    }

    let pending = client
        .eth_get_transaction_count(from.into(), "pending".to_string())
        .await
        .unwrap();
    assert_eq!(pending, "0x2");
    let latest = client
        .eth_get_transaction_count(from.into(), "latest".to_string())
        .await
        .unwrap();
    assert_eq!(latest, "0x0");

    server.stop().unwrap();
}

#[tokio::test]
async fn transact_removes_tx_from_pool() {
    let (server, client) = spawn_test_server(Default::default()).await;
    let wallet = EthereumWallet::new(PrivateKeySigner::random());
    let deploy_data =
        Bytes::from_str(&load_file_as_string("brc20_prog_helper_deploy_tx_data").unwrap()).unwrap();
    let tx = sign_legacy(&wallet, chain_id(&client).await, 0, U256::ZERO, deploy_data).await;

    let hash = client.eth_send_raw_transaction(raw(&tx)).await.unwrap();
    assert!(client
        .eth_get_raw_transaction_by_hash(hash.clone())
        .await
        .unwrap()
        .is_some());

    let receipts = client
        .brc20_transact(
            raw(&tx).into(),
            None,
            42,
            [0u8; 32].into(),
            0,
            "deploy_inscription".to_string(),
            tx.len() as u64,
            [1; 32].into(),
        )
        .await
        .unwrap();
    assert_eq!(receipts.len(), 1);

    assert!(client
        .eth_get_raw_transaction_by_hash(hash)
        .await
        .unwrap()
        .is_none());

    server.stop().unwrap();
}

#[tokio::test]
async fn pool_methods_are_public() {
    let db_path = TempDir::new().unwrap();
    let public_address = format!("127.0.0.1:{}", get_free_port());
    let admin_address = format!("127.0.0.1:{}", get_free_port());
    let handle = start(Brc20ProgConfig {
        db_path: db_path.path().to_str().unwrap().to_string(),
        brc20_prog_rpc_server_url: public_address.clone(),
        brc20_prog_admin_rpc_server_url: Some(admin_address),
        brc20_prog_rpc_server_enable_auth: true,
        brc20_prog_rpc_server_user: Some("user".to_string()),
        brc20_prog_rpc_server_password: Some("password".to_string()),
        fail_on_bitcoin_rpc_error: false,
        ..Default::default()
    })
    .await
    .expect("Failed to start server");
    let public = HttpClientBuilder::default()
        .build(format!("http://{}", public_address))
        .unwrap();

    let wallet = EthereumWallet::new(PrivateKeySigner::random());
    let tx = sign_legacy(
        &wallet,
        chain_id(&public).await,
        0,
        U256::ZERO,
        Bytes::new(),
    )
    .await;

    let hash = public.eth_send_raw_transaction(raw(&tx)).await.unwrap();
    assert!(public
        .eth_get_raw_transaction_by_hash(hash)
        .await
        .unwrap()
        .is_some());
    assert_eq!(
        public.brc20_unsent_raw_transactions().await.unwrap().len(),
        1
    );

    handle.stop().unwrap();
}
