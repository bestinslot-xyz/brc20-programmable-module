use std::error::Error;
use std::str::FromStr;

use alloy::primitives::{Bytes, U256};
use alloy_consensus::TxLegacy;
use alloy_network::{EthereumWallet, TransactionBuilder};
use alloy_rpc_types_eth::TransactionRequest;
use alloy_signer_local::PrivateKeySigner;
use brc20_prog::types::{Base64Bytes, RawBytes};
use brc20_prog::Brc20ProgApiClient;
use revm::primitives::Address;
use test_utils::{load_file_as_string, spawn_test_server};

#[tokio::test]
async fn test_transact() -> Result<(), Box<dyn Error>> {
    let (server, client) = spawn_test_server(Default::default()).await;
    let from_pkscript = "7465737420706b736372697074".to_string(); // "test pkscript"
    let timestamp = 42;
    let block_hash = [0u8; 32].into();

    let chain_id = client.eth_chain_id().await.unwrap();
    let chain_id_number = u64::from_str_radix(chain_id.trim_start_matches("0x"), 16)?;

    let wallet = EthereumWallet::new(PrivateKeySigner::random());

    let deploy_data = load_file_as_string("brc20_prog_helper_deploy_tx_data")?;
    let tx_builder: TransactionRequest = TxLegacy::default().into();
    let tx_builder = tx_builder
        .with_chain_id(chain_id_number)
        .with_nonce(0)
        .with_gas_price(0)
        .with_gas_limit(0)
        .with_to(Address::ZERO)
        .with_value(U256::ZERO)
        .with_input(Bytes::from_str(&deploy_data).unwrap());

    let built_tx = tx_builder.build(&wallet).await.unwrap();

    let signed_tx = built_tx.into_signed();

    let mut rlp_encoded = Vec::new();
    signed_tx.network_encode(&mut rlp_encoded);
    let deploy_data_length = rlp_encoded.len() as u64;

    let deploy_response = client
        .brc20_transact(
            RawBytes::new(hex::encode(rlp_encoded)).into(),
            None,
            timestamp,
            block_hash,
            0,
            Some("deploy_inscription".to_string()),
            deploy_data_length.into(),
        )
        .await?;

    let tx_hash = deploy_response.first().unwrap().transaction_hash;

    let receipt = client.eth_get_transaction_receipt(tx_hash).await?.unwrap();

    let contract_address = receipt.contract_address.unwrap();

    let call_data = load_file_as_string("brc20_prog_helper_call_tx_data")?;
    let call_data_length = call_data.len() as u64;
    let call_response = client
        .brc20_call(
            from_pkscript,
            contract_address.into(),
            None,
            RawBytes::new(call_data.to_string()).into(),
            None,
            timestamp,
            block_hash,
            1,
            Some("call_inscription1".to_string()),
            call_data_length.into(),
        )
        .await?
        .unwrap();

    assert_eq!(
        call_response.result_bytes.unwrap().bytes,
        Bytes::from_str(&load_file_as_string("brc20_prog_helper_call_response")?).unwrap()
    );

    let call_tx_builder: TransactionRequest = TxLegacy::default().into();
    let call_tx_builder = call_tx_builder
        .with_chain_id(chain_id_number)
        .with_nonce(1)
        .with_gas_price(0)
        .with_gas_limit(0)
        .with_to(contract_address.address)
        .with_value(U256::ZERO)
        .with_input(Bytes::from_str(&call_data).unwrap());

    let built_tx = call_tx_builder.build(&wallet).await.unwrap();

    let signed_tx = built_tx.into_signed();

    let mut rlp_encoded = Vec::new();
    signed_tx.network_encode(&mut rlp_encoded);
    let call_data_length = rlp_encoded.len() as u64;

    let call_response = client
        .brc20_transact(
            RawBytes::new(hex::encode(rlp_encoded)).into(),
            None,
            timestamp,
            block_hash,
            2,
            Some("call_inscription2".to_string()),
            call_data_length.into(),
        )
        .await?;

    let receipt = call_response.first().unwrap().clone();

    assert_eq!(
        receipt.result_bytes.unwrap().bytes,
        Bytes::from_str(&load_file_as_string("brc20_prog_helper_call_response")?).unwrap()
    );

    server.stop()?;

    Ok(())
}

#[tokio::test]
async fn test_transact_encoded() -> Result<(), Box<dyn Error>> {
    let (server, client) = spawn_test_server(Default::default()).await;
    let from_pkscript = "7465737420706b736372697074".to_string(); // "test pkscript"
    let timestamp = 42;
    let block_hash = [0u8; 32].into();

    let chain_id = client.eth_chain_id().await.unwrap();
    let chain_id_number = u64::from_str_radix(chain_id.trim_start_matches("0x"), 16)?;

    let wallet = EthereumWallet::new(PrivateKeySigner::random());

    let deploy_data = load_file_as_string("brc20_prog_helper_deploy_tx_data")?;
    let tx_builder: TransactionRequest = TxLegacy::default().into();
    let tx_builder = tx_builder
        .with_chain_id(chain_id_number)
        .with_nonce(0)
        .with_gas_price(0)
        .with_gas_limit(0)
        .with_to(Address::ZERO)
        .with_value(U256::ZERO)
        .with_input(Bytes::from_str(&deploy_data).unwrap());

    let built_tx = tx_builder.build(&wallet).await.unwrap();

    let signed_tx = built_tx.into_signed();

    let mut rlp_encoded = Vec::new();
    signed_tx.network_encode(&mut rlp_encoded);
    let deploy_data_length = rlp_encoded.len() as u64;

    let deploy_response = client
        .brc20_transact(
            None,
            Base64Bytes::from_bytes(rlp_encoded.into()).unwrap().into(),
            timestamp,
            block_hash,
            0,
            Some("deploy_inscription".to_string()),
            deploy_data_length.into(),
        )
        .await
        .unwrap();

    let receipt = deploy_response.first().unwrap();

    let contract_address = receipt.contract_address.unwrap();

    let call_data = load_file_as_string("brc20_prog_helper_call_tx_data")?;
    let call_data_length = call_data.len() as u64;
    let call_response = client
        .brc20_call(
            from_pkscript,
            contract_address.into(),
            None,
            None,
            Base64Bytes::from_bytes(Bytes::from_str(&call_data).unwrap())
                .unwrap()
                .into(),
            timestamp,
            block_hash,
            1,
            Some("call_inscription1".to_string()),
            call_data_length.into(),
        )
        .await?
        .unwrap();

    assert_eq!(
        call_response.result_bytes.unwrap().bytes,
        Bytes::from_str(&load_file_as_string("brc20_prog_helper_call_response")?).unwrap()
    );

    let call_tx_builder: TransactionRequest = TxLegacy::default().into();
    let call_tx_builder = call_tx_builder
        .with_chain_id(chain_id_number)
        .with_nonce(1)
        .with_gas_price(0)
        .with_gas_limit(0)
        .with_to(contract_address.address)
        .with_value(U256::ZERO)
        .with_input(Bytes::from_str(&call_data).unwrap());

    let built_tx = call_tx_builder.build(&wallet).await.unwrap();

    let signed_tx = built_tx.into_signed();

    let mut rlp_encoded = Vec::new();
    signed_tx.network_encode(&mut rlp_encoded);
    let call_data_length = rlp_encoded.len() as u64;

    let call_response = client
        .brc20_transact(
            None,
            Base64Bytes::from_bytes(rlp_encoded.into()).unwrap().into(),
            timestamp,
            block_hash,
            2,
            Some("call_inscription2".to_string()),
            call_data_length.into(),
        )
        .await?;

    let receipt = call_response.first().unwrap().clone();

    assert_eq!(
        receipt.result_bytes.unwrap().bytes,
        Bytes::from_str(&load_file_as_string("brc20_prog_helper_call_response")?).unwrap()
    );

    server.stop()?;

    Ok(())
}

#[tokio::test]
async fn test_transact_out_of_order() -> Result<(), Box<dyn Error>> {
    let (server, client) = spawn_test_server(Default::default()).await;
    let timestamp = 42;
    let block_hash = [0u8; 32].into();

    let chain_id = client.eth_chain_id().await.unwrap();
    let chain_id_number = u64::from_str_radix(chain_id.trim_start_matches("0x"), 16)?;

    let wallet = EthereumWallet::new(PrivateKeySigner::random());

    let deploy_data = load_file_as_string("brc20_prog_helper_deploy_tx_data")?;
    let tx_builder: TransactionRequest = TxLegacy::default().into();
    let tx_builder = tx_builder
        .with_chain_id(chain_id_number)
        .with_nonce(0)
        .with_gas_price(0)
        .with_gas_limit(0)
        .with_to(Address::ZERO)
        .with_value(U256::ZERO)
        .with_input(Bytes::from_str(&deploy_data).unwrap());

    let built_tx = tx_builder.build(&wallet).await.unwrap();

    let signed_tx = built_tx.into_signed();

    let mut rlp_encoded = Vec::new();
    signed_tx.network_encode(&mut rlp_encoded);
    let deploy_data_length = rlp_encoded.len() as u64;

    let deploy_response = client
        .brc20_transact(
            None,
            Base64Bytes::from_bytes(rlp_encoded.into()).unwrap().into(),
            timestamp,
            block_hash,
            0,
            Some("deploy_inscription".to_string()),
            deploy_data_length.into(),
        )
        .await
        .unwrap();

    let receipt = deploy_response.first().unwrap();

    let contract_address = receipt.contract_address.unwrap();

    let call_data = load_file_as_string("brc20_prog_helper_call_tx_data")?;

    let call_tx_builder: TransactionRequest = TxLegacy::default().into();
    let call_tx_builder = call_tx_builder
        .with_chain_id(chain_id_number)
        .with_nonce(2)
        .with_gas_price(0)
        .with_gas_limit(0)
        .with_to(contract_address.address)
        .with_value(U256::ZERO)
        .with_input(Bytes::from_str(&call_data).unwrap());

    let built_tx = call_tx_builder.build(&wallet).await.unwrap();

    let signed_tx = built_tx.into_signed();

    let mut rlp_encoded = Vec::new();
    signed_tx.network_encode(&mut rlp_encoded);
    let call_data_length = rlp_encoded.len() as u64;

    let call_response = client
        .brc20_transact(
            None,
            Base64Bytes::from_bytes(rlp_encoded.into()).unwrap().into(),
            timestamp,
            block_hash,
            2,
            Some("call_inscription".to_string()),
            call_data_length.into(),
        )
        .await
        .unwrap();

    assert!(call_response.is_empty()); // Nonce 2 should not be processed before nonce 1

    let call_tx_builder: TransactionRequest = TxLegacy::default().into();
    let call_tx_builder = call_tx_builder
        .with_chain_id(chain_id_number)
        .with_nonce(3)
        .with_gas_price(0)
        .with_gas_limit(0)
        .with_to(contract_address.address)
        .with_value(U256::ZERO)
        .with_input(Bytes::from_str(&call_data).unwrap());

    let built_tx = call_tx_builder.build(&wallet).await.unwrap();

    let signed_tx = built_tx.into_signed();

    let mut rlp_encoded = Vec::new();
    signed_tx.network_encode(&mut rlp_encoded);
    let call_data_length = rlp_encoded.len() as u64;

    let call_response = client
        .brc20_transact(
            None,
            Base64Bytes::from_bytes(rlp_encoded.into()).unwrap().into(),
            timestamp,
            block_hash,
            2,
            Some("call_inscription3".to_string()),
            call_data_length.into(),
        )
        .await?;

    assert!(call_response.is_empty()); // Nonce 3 should not be processed before nonce 1

    let call_tx_builder: TransactionRequest = TxLegacy::default().into();
    let call_tx_builder = call_tx_builder
        .with_chain_id(chain_id_number)
        .with_nonce(1)
        .with_gas_price(0)
        .with_gas_limit(0)
        .with_to(contract_address.address)
        .with_value(U256::ZERO)
        .with_input(Bytes::from_str(&call_data).unwrap());

    let built_tx = call_tx_builder.build(&wallet).await.unwrap();

    let signed_tx = built_tx.into_signed();

    let mut rlp_encoded = Vec::new();
    signed_tx.network_encode(&mut rlp_encoded);
    let call_data_length = rlp_encoded.len() as u64;

    let call_response = client
        .brc20_transact(
            None,
            Base64Bytes::from_bytes(rlp_encoded.into()).unwrap().into(),
            timestamp,
            block_hash,
            1,
            Some("call_inscription2".to_string()),
            call_data_length.into(),
        )
        .await?;

    assert_eq!(call_response.len(), 3); // Now nonce 1 and 2 should be processed together

    let receipt = call_response.get(0).unwrap().clone();

    assert_eq!(receipt.nonce, 1u64.into());
    assert_eq!(
        receipt.result_bytes.unwrap().bytes,
        Bytes::from_str(&load_file_as_string("brc20_prog_helper_call_response")?).unwrap()
    );

    let receipt = call_response.get(1).unwrap().clone();

    assert_eq!(receipt.nonce, 2u64.into());
    assert_eq!(
        receipt.result_bytes.unwrap().bytes,
        Bytes::from_str(&load_file_as_string("brc20_prog_helper_call_response")?).unwrap()
    );

    let receipt = call_response.get(2).unwrap().clone();

    assert_eq!(receipt.nonce, 3u64.into());
    assert_eq!(
        receipt.result_bytes.unwrap().bytes,
        Bytes::from_str(&load_file_as_string("brc20_prog_helper_call_response")?).unwrap()
    );

    server.stop()?;

    Ok(())
}

#[tokio::test]
async fn test_transact_remove_old_transactions() -> Result<(), Box<dyn Error>> {
    let (server, client) = spawn_test_server(Default::default()).await;
    let timestamp = 42;
    let block_hash = [0u8; 32].into();

    let chain_id = client.eth_chain_id().await.unwrap();
    let chain_id_number = u64::from_str_radix(chain_id.trim_start_matches("0x"), 16)?;

    let signer = PrivateKeySigner::random();
    let address = signer.address();
    let wallet = EthereumWallet::new(signer);

    let deploy_data = load_file_as_string("brc20_prog_helper_deploy_tx_data")?;
    let tx_builder: TransactionRequest = TxLegacy::default().into();
    let tx_builder = tx_builder
        .with_chain_id(chain_id_number)
        .with_nonce(1)
        .with_gas_price(0)
        .with_gas_limit(0)
        .with_to(Address::ZERO)
        .with_value(U256::ZERO)
        .with_input(Bytes::from_str(&deploy_data).unwrap());

    let built_tx = tx_builder.build(&wallet).await.unwrap();

    let signed_tx = built_tx.into_signed();

    let mut rlp_encoded = Vec::new();
    signed_tx.network_encode(&mut rlp_encoded);
    let deploy_data_length = rlp_encoded.len() as u64;

    let deploy_response = client
        .brc20_transact(
            None,
            Base64Bytes::from_bytes(rlp_encoded.into()).unwrap().into(),
            timestamp,
            block_hash,
            0,
            Some("deploy_inscription".to_string()),
            deploy_data_length.into(),
        )
        .await
        .unwrap();

    assert_eq!(deploy_response.len(), 0); // No transactions should be processed yet

    let txpool = client.txpool_content().await?;
    assert!(txpool
        .get("pending")
        .unwrap()
        .get(&address.into())
        .unwrap()
        .get(&1)
        .is_some());

    // Generate and send 10 blocks to invalidate the old transactions
    for _ in 0..11 {
        client
            .brc20_finalise_block(timestamp, [0u8; 32].into(), 0)
            .await?;
    }

    let txpool = client.txpool_content().await?;
    assert!(txpool
        .get("pending")
        .unwrap()
        .get(&address.into())
        .is_none());

    server.stop()?;

    Ok(())
}

#[tokio::test]
async fn test_transact_in_the_past() -> Result<(), Box<dyn Error>> {
    let (server, client) = spawn_test_server(Default::default()).await;
    let timestamp = 42;
    let block_hash = [0u8; 32].into();

    let chain_id = client.eth_chain_id().await.unwrap();
    let chain_id_number = u64::from_str_radix(chain_id.trim_start_matches("0x"), 16)?;

    let signer = PrivateKeySigner::random();
    let wallet = EthereumWallet::new(signer);

    let deploy_data = load_file_as_string("brc20_prog_helper_deploy_tx_data")?;
    let tx_builder: TransactionRequest = TxLegacy::default().into();
    let tx_builder = tx_builder
        .with_chain_id(chain_id_number)
        .with_nonce(0)
        .with_gas_price(0)
        .with_gas_limit(0)
        .with_to(Address::ZERO)
        .with_value(U256::ZERO)
        .with_input(Bytes::from_str(&deploy_data).unwrap());

    let built_tx = tx_builder.build(&wallet).await.unwrap();

    let signed_tx = built_tx.into_signed();

    let mut rlp_encoded = Vec::new();
    signed_tx.network_encode(&mut rlp_encoded);
    let deploy_data_length = rlp_encoded.len() as u64;

    let deploy_response = client
        .brc20_transact(
            None,
            Base64Bytes::from_bytes(rlp_encoded.into()).unwrap().into(),
            timestamp,
            block_hash,
            0,
            Some("deploy_inscription".to_string()),
            deploy_data_length.into(),
        )
        .await
        .unwrap();

    assert_eq!(deploy_response.len(), 1);

    let txpool = client.txpool_content().await?;
    assert!(txpool.get("pending").unwrap().is_empty()); // No transactions in the pool, as it's processed immediately

    let tx_builder: TransactionRequest = TxLegacy::default().into();
    let tx_builder = tx_builder
        .with_chain_id(chain_id_number)
        .with_nonce(0)
        .with_gas_price(0)
        .with_gas_limit(0)
        .with_to(Address::ZERO)
        .with_value(U256::ZERO)
        .with_input(Bytes::from_str(&deploy_data).unwrap());

    let built_tx = tx_builder.build(&wallet).await.unwrap();

    let signed_tx = built_tx.into_signed();

    let mut rlp_encoded = Vec::new();
    signed_tx.network_encode(&mut rlp_encoded);
    let deploy_data_length = rlp_encoded.len() as u64;

    let deploy_response = client
        .brc20_transact(
            None,
            Base64Bytes::from_bytes(rlp_encoded.into()).unwrap().into(),
            timestamp,
            block_hash,
            0,
            Some("deploy_inscription".to_string()),
            deploy_data_length.into(),
        )
        .await
        .unwrap();

    assert_eq!(deploy_response.len(), 0); // Transaction should be ignored due to past nonce

    let txpool = client.txpool_content().await?;
    assert!(txpool.get("pending").unwrap().is_empty()); // No transactions in the pool

    server.stop()?;

    Ok(())
}

#[tokio::test]
async fn test_transact_in_the_future() -> Result<(), Box<dyn Error>> {
    let (server, client) = spawn_test_server(Default::default()).await;
    let timestamp = 42;
    let block_hash = [0u8; 32].into();

    let chain_id = client.eth_chain_id().await.unwrap();
    let chain_id_number = u64::from_str_radix(chain_id.trim_start_matches("0x"), 16)?;

    let signer = PrivateKeySigner::random();
    let wallet = EthereumWallet::new(signer);

    let deploy_data = load_file_as_string("brc20_prog_helper_deploy_tx_data")?;
    let tx_builder: TransactionRequest = TxLegacy::default().into();
    let tx_builder = tx_builder
        .with_chain_id(chain_id_number)
        .with_nonce(11)
        .with_gas_price(0)
        .with_gas_limit(0)
        .with_to(Address::ZERO)
        .with_value(U256::ZERO)
        .with_input(Bytes::from_str(&deploy_data).unwrap());

    let built_tx = tx_builder.build(&wallet).await.unwrap();

    let signed_tx = built_tx.into_signed();

    let mut rlp_encoded = Vec::new();
    signed_tx.network_encode(&mut rlp_encoded);
    let deploy_data_length = rlp_encoded.len() as u64;

    let deploy_response = client
        .brc20_transact(
            None,
            Base64Bytes::from_bytes(rlp_encoded.into()).unwrap().into(),
            timestamp,
            block_hash,
            0,
            Some("deploy_inscription".to_string()),
            deploy_data_length.into(),
        )
        .await
        .unwrap();

    assert_eq!(deploy_response.len(), 0); // Transaction should be ignored due to future nonce

    let txpool = client.txpool_content().await?;
    assert!(txpool.get("pending").unwrap().is_empty()); // No transactions in the pool

    server.stop()?;

    Ok(())
}
