use std::error::Error;
use std::str::FromStr;

use alloy::primitives::Bytes;
use brc20_prog::types::{Base64Bytes, RawBytes};
use brc20_prog::Brc20ProgApiClient;
use test_utils::{load_file_as_string, spawn_test_server};

#[tokio::test]
async fn test_deploy_call() -> Result<(), Box<dyn Error>> {
    let (server, client) = spawn_test_server(Default::default()).await;
    let from_pkscript = "7465737420706b736372697074".to_string(); // "test pkscript"
    let timestamp = 42;
    let block_hash = [0u8; 32].into();

    let deploy_data = load_file_as_string("brc20_prog_helper_deploy_tx_data")?;
    let deploy_data_length = deploy_data.len() as u64;
    let deploy_response = client
        .brc20_deploy(
            from_pkscript.clone(),
            RawBytes::new(deploy_data).into(),
            None,
            timestamp,
            block_hash,
            0,
            "deploy_inscription".to_string(),
            deploy_data_length,
            [1; 32].into(),
        )
        .await?;

    let contract_address = deploy_response.contract_address.unwrap();

    let call_data = load_file_as_string("brc20_prog_helper_call_tx_data")?;
    let call_data_length = call_data.len() as u64;
    let call_response = client
        .brc20_call(
            from_pkscript,
            contract_address.into(),
            None,
            RawBytes::new(call_data).into(),
            None,
            timestamp,
            block_hash,
            1,
            "call_inscription".to_string(),
            call_data_length,
            [2; 32].into(),
        )
        .await?
        .unwrap();

    assert!(!call_response.status.is_zero());

    let trace = client
        .debug_trace_transaction(call_response.transaction_hash)
        .await?;

    assert_eq!(
        trace.unwrap().output.bytes,
        Bytes::from_str(&load_file_as_string("brc20_prog_helper_call_response")?).unwrap()
    );

    server.stop()?;

    Ok(())
}

#[tokio::test]
async fn test_deploy_call_encoded() -> Result<(), Box<dyn Error>> {
    let (server, client) = spawn_test_server(Default::default()).await;
    let from_pkscript = "7465737420706b736372697074".to_string(); // "test pkscript"
    let timestamp = 42;
    let block_hash = [0u8; 32].into();

    let deploy_data = load_file_as_string("brc20_prog_helper_deploy_tx_data")?;
    let deploy_data_length = deploy_data.len() as u64;
    let deploy_response = client
        .brc20_deploy(
            from_pkscript.clone(),
            None,
            Base64Bytes::from_bytes(Bytes::from_str(&deploy_data).unwrap())
                .unwrap()
                .into(),
            timestamp,
            block_hash,
            0,
            "deploy_inscription".to_string(),
            deploy_data_length,
            [1; 32].into(),
        )
        .await?;

    let contract_address = deploy_response.contract_address.unwrap();

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
            "call_inscription".to_string(),
            call_data_length,
            [2; 32].into(),
        )
        .await?
        .unwrap();

    assert!(!call_response.status.is_zero());

    let trace = client
        .debug_trace_transaction(call_response.transaction_hash)
        .await?;

    assert_eq!(
        trace.unwrap().output.bytes,
        Bytes::from_str(&load_file_as_string("brc20_prog_helper_call_response")?).unwrap()
    );
    server.stop()?;

    Ok(())
}
