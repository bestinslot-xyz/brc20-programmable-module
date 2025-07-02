use std::error::Error;

use brc20_prog::types::EthCall;
use brc20_prog::{Brc20ProgApiClient, Brc20ProgConfig};
use test_utils::{
    is_in_ci, load_file_as_eth_bytes, load_file_as_string, spawn_balance_server, spawn_test_server,
};

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

    server.stop()?;

    Ok(())
}

async fn verify_btc_last_sat_loc_signet() -> Result<(), Box<dyn Error>> {
    if is_in_ci() {
        return Ok(());
    }

    dotenvy::from_filename_override(".env.signet").ok();

    let (server, client) = spawn_test_server(Default::default()).await;

    let mut btc_last_sat_loc_precompile = [0; 20];
    btc_last_sat_loc_precompile[19] = 0xfc;

    // Mine some blocks to ensure the transaction is included in a block
    client.brc20_mine(250000, 0).await?;

    let response = client
        .eth_call(
            EthCall::new(
                Some([1u8; 20].into()),
                Some(btc_last_sat_loc_precompile.into()),
                load_file_as_eth_bytes("btc_last_sat_loc_signet_call_tx_data")?,
            ),
            Some("latest".to_string()),
        )
        .await?;

    assert_eq!(
        response,
        load_file_as_string("btc_last_sat_loc_signet_call_response")?
    );

    server.stop()?;
    Ok(())
}

async fn verify_btc_get_tx_details(
    envfile: &str,
    call_file: &str,
    response_file: &str,
) -> Result<(), Box<dyn Error>> {
    if is_in_ci() {
        return Ok(());
    }

    dotenvy::from_filename_override(envfile).ok();

    let (server, client) = spawn_test_server(Default::default()).await;

    let mut btc_get_tx_details_precompile = [0; 20];
    btc_get_tx_details_precompile[19] = 0xfd;

    // Mine some blocks to ensure the transaction is included in a block
    client.brc20_mine(300000, 0).await?;

    let response = client
        .eth_call(
            EthCall::new(
                Some([1u8; 20].into()),
                Some(btc_get_tx_details_precompile.into()),
                load_file_as_eth_bytes(call_file)?,
            ),
            Some("latest".to_string()),
        )
        .await?;

    assert_eq!(response, load_file_as_string(response_file)?);

    server.stop()?;
    Ok(())
}

#[tokio::test]
async fn test_btc_rpc_precompiles() -> Result<(), Box<dyn Error>> {
    if is_in_ci() {
        return Ok(());
    }

    // Precompiles that interact with Bitcoin RPC are run sequentially
    // as environment variables (for mainnet and signet) need to be reloaded between runs.
    verify_btc_get_tx_details(
        ".env.mainnet",
        "btc_get_tx_details_mainnet_call_1_tx_data",
        "btc_get_tx_details_mainnet_call_1_response",
    )
    .await
    .expect("Failed to verify btc_get_tx_details mainnet call 1");
    verify_btc_get_tx_details(
        ".env.mainnet",
        "btc_get_tx_details_mainnet_call_2_tx_data",
        "btc_get_tx_details_mainnet_call_2_response",
    )
    .await
    .expect("Failed to verify btc_get_tx_details mainnet call 2");
    verify_btc_get_tx_details(
        ".env.mainnet",
        "btc_get_tx_details_mainnet_call_3_tx_data",
        "btc_get_tx_details_mainnet_call_3_response",
    )
    .await
    .expect("Failed to verify btc_get_tx_details mainnet call 3");

    verify_btc_get_tx_details(
        ".env.signet",
        "btc_get_tx_details_signet_call_tx_data",
        "btc_get_tx_details_signet_call_response",
    )
    .await
    .expect("Failed to verify btc_get_tx_details signet call");
    verify_btc_last_sat_loc_signet()
        .await
        .expect("Failed to verify btc_last_sat_loc signet call");

    Ok(())
}

#[tokio::test]
async fn test_get_brc20_balance() -> Result<(), Box<dyn Error>> {
    let port = spawn_balance_server();

    // Wait for the server to start
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let mut get_brc20_balance_precompile = [0; 20];
    get_brc20_balance_precompile[19] = 0xff;

    let call_tx_data = load_file_as_eth_bytes("get_brc20_balance_call_tx_data")?;
    let call_response = load_file_as_string("get_brc20_balance_call_response")?;

    // Wait for the server to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let (server, client) = spawn_test_server(Brc20ProgConfig {
        brc20_balance_server_url: format!("http://localhost:{}", port),
        ..Default::default()
    })
    .await;
    let response = client
        .eth_call(
            EthCall::new(
                Some([1u8; 20].into()),
                Some(get_brc20_balance_precompile.into()),
                call_tx_data,
            ),
            Some("latest".to_string()),
        )
        .await
        .unwrap();

    assert_eq!(response, call_response);

    server.stop().unwrap();
    Ok(())
}
