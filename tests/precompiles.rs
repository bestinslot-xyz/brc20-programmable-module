use std::error::Error;

use brc20_prog::types::{EthCall, RawBytes};
use brc20_prog::{Brc20ProgApiClient, Brc20ProgConfig};
use revm::primitives::U256;
use test_utils::{load_file_as_eth_bytes, load_file_as_string, spawn_test_server};

#[tokio::test]
async fn test_current_tx_id_before_prague() -> Result<(), Box<dyn Error>> {
    let (server, client) = spawn_test_server(Brc20ProgConfig {
        bitcoin_rpc_network: "signet".to_string(),
        ..Default::default()
    })
    .await;
    let mut current_tx_id_precompile = [0; 20];
    current_tx_id_precompile[19] = 0xfa;

    // Mine less than prague activation height
    client.brc20_mine(274999, 0).await?;

    let response = client
        .brc20_call(
            "aaaaabbbbccccddddeeeeffff00001111222233333".to_string(),
            Some(current_tx_id_precompile.into()),
            None,
            Some(RawBytes::new("0x00".to_string())),
            None,
            12345,
            [0u8; 32].into(),
            0,
            "inscription".to_string(),
            100,
            [5u8; 32].into(),
        )
        .await?
        .unwrap();

    let trace = client
        .debug_trace_transaction(response.transaction_hash)
        .await?
        .unwrap();

    assert_eq!(trace.output, vec![].into());

    server.stop()?;

    Ok(())
}

#[tokio::test]
async fn test_current_tx_id() -> Result<(), Box<dyn Error>> {
    let (server, client) = spawn_test_server(Brc20ProgConfig {
        bitcoin_rpc_network: "signet".to_string(),
        ..Default::default()
    })
    .await;
    let mut current_tx_id_precompile = [0; 20];
    current_tx_id_precompile[19] = 0xfa;

    // Mine some blocks to ensure we hit prague activation height
    client.brc20_mine(275000, 0).await?;

    let block_number = client.eth_block_number().await?;
    assert_eq!(block_number.as_str(), "0x43237"); // 275000 - 1 in hex

    let response = client
        .brc20_call(
            "aaaaabbbbccccddddeeeeffff00001111222233333".to_string(),
            Some(current_tx_id_precompile.into()),
            None,
            Some(RawBytes::new("0x00".to_string())),
            None,
            12345,
            [0u8; 32].into(),
            0,
            "inscription".to_string(),
            100,
            [5u8; 32].into(),
        )
        .await?
        .unwrap();

    let trace = client
        .debug_trace_transaction(response.transaction_hash)
        .await?
        .unwrap();

    assert_eq!(trace.gas_used.uint, U256::from(21044));

    assert_eq!(trace.output, [5u8; 32].to_vec().into());

    server.stop()?;

    Ok(())
}

#[tokio::test]
async fn test_bip322_verify() -> Result<(), Box<dyn Error>> {
    let (server, client) = spawn_test_server(Default::default()).await;
    let mut bip322_precompile = [0; 20];
    bip322_precompile[19] = 0xfe;

    let response = client
        .eth_call(
            EthCall::new(
                Some([1u8; 20].into()),
                Some(bip322_precompile.into()),
                load_file_as_eth_bytes("bip322_verify_call_tx_data")?,
            ),
            Some("latest".to_string()),
        )
        .await?;

    assert_eq!(
        response,
        "0x0000000000000000000000000000000000000000000000000000000000000001"
    );

    let gas_response = client.eth_estimate_gas(
        EthCall::new(
            Some([1u8; 20].into()),
            Some(bip322_precompile.into()),
            load_file_as_eth_bytes("bip322_verify_call_tx_data")?,
        ),
        Some("latest".to_string()),
    ).await?;

    assert_eq!(gas_response.as_str().to_lowercase(), "0xc93c");

    server.stop()?;

    Ok(())
}

#[tokio::test]
async fn test_btc_locked_pkscript() -> Result<(), Box<dyn Error>> {
    let (server, client) = spawn_test_server(Default::default()).await;

    let mut btc_locked_pkscript_precompile = [0; 20];
    btc_locked_pkscript_precompile[19] = 0xfb;

    let response = client
        .eth_call(
            EthCall::new(
                Some([1u8; 20].into()),
                Some(btc_locked_pkscript_precompile.into()),
                load_file_as_eth_bytes("btc_get_locked_pkscript_call_tx_data")?,
            ),
            Some("latest".to_string()),
        )
        .await?;

    assert_eq!(
        response,
        load_file_as_string("btc_get_locked_pkscript_call_response")?
    );

    let gas_response = client.eth_estimate_gas(
        EthCall::new(
            Some([1u8; 20].into()),
            Some(btc_locked_pkscript_precompile.into()),
            load_file_as_eth_bytes("btc_get_locked_pkscript_call_tx_data")?,
        ),
        Some("latest".to_string()),
    ).await?;

    assert_eq!(gas_response.as_str().to_lowercase(), "0xab6f");

    server.stop()?;

    Ok(())
}
