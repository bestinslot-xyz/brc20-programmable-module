use std::error::Error;
use std::str::FromStr;

use alloy::primitives::{Bytes, U256};
use alloy_consensus::TxLegacy;
use alloy_network::{EthereumWallet, TransactionBuilder};
use alloy_rpc_types_eth::TransactionRequest;
use alloy_signer_local::PrivateKeySigner;
use brc20_prog::types::InscriptionBytes;
use brc20_prog::{Brc20ProgApiClient, Brc20ProgConfig};
use revm::primitives::Address;
use test_utils::{load_file_as_string, spawn_test_server};

#[tokio::test]
async fn test_transact() -> Result<(), Box<dyn Error>> {
    let (server, client) = spawn_test_server(Brc20ProgConfig {
        brc20_transact_endpoint_enabled: true,
        ..Default::default()
    })
    .await;
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
            InscriptionBytes::new(hex::encode(rlp_encoded)),
            timestamp,
            block_hash,
            0,
            Some("deploy_inscription".to_string()),
            deploy_data_length.into(),
        )
        .await?;

    let tx_hash = deploy_response.unwrap().hash;

    let receipt = client.eth_get_transaction_receipt(tx_hash).await?.unwrap();

    let contract_address = receipt.contract_address.unwrap();

    let call_data = load_file_as_string("brc20_prog_helper_call_tx_data")?;
    let call_data_length = call_data.len() as u64;
    let call_response = client
        .brc20_call(
            from_pkscript,
            contract_address.into(),
            None,
            InscriptionBytes::new(call_data.to_string()),
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
        .with_nonce(0)
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
            InscriptionBytes::new(hex::encode(rlp_encoded)),
            timestamp,
            block_hash,
            2,
            Some("call_inscription2".to_string()),
            call_data_length.into(),
        )
        .await?
        .unwrap();

    let tx_hash = call_response.hash;

    let call_response = client.eth_get_transaction_receipt(tx_hash).await?.unwrap();

    assert_eq!(
        call_response.result_bytes.unwrap().bytes,
        Bytes::from_str(&load_file_as_string("brc20_prog_helper_call_response")?).unwrap()
    );

    server.stop()?;

    Ok(())
}
