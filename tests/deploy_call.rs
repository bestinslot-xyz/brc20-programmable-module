use std::error::Error;
use std::str::FromStr;

use alloy_primitives::Bytes;
use brc20_prog::types::EncodedBytes;
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
            EncodedBytes::new(deploy_data),
            timestamp,
            block_hash,
            0,
            Some("deploy_inscription".to_string()),
            deploy_data_length.into(),
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
            EncodedBytes::new(call_data),
            timestamp,
            block_hash,
            1,
            Some("call_inscription".to_string()),
            call_data_length.into(),
        )
        .await?
        .unwrap();

    assert_eq!(
        call_response.result_bytes.unwrap().bytes,
        Bytes::from_str(&load_file_as_string("brc20_prog_helper_call_response")?).unwrap()
    );

    server.stop()?;

    Ok(())
}
