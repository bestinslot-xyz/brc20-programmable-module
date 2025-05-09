mod common;
use std::error::Error;

use brc20_prog::{
    types::EthCall,
    Brc20ProgApiClient,
};
use common::{is_in_ci, load_file_as_bytes, load_file_as_string, spawn_test_server};

#[tokio::test]
async fn test_bip322_verify() -> Result<(), Box<dyn Error>> {
    let (server, client) = spawn_test_server().await;

    let mut bip322_precompile = [0; 20];
    bip322_precompile[19] = 0xfe;

    let response = client
        .call(
            EthCall::new(
                Some([1u8; 20].into()),
                Some(bip322_precompile.into()),
                load_file_as_bytes("bip322_verify_call_tx_data")?,
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
    let (server, client) = spawn_test_server().await;

    let mut btc_locked_pkscript_precompile = [0; 20];
    btc_locked_pkscript_precompile[19] = 0xfb;

    let response = client
        .call(
            EthCall::new(
                Some([1u8; 20].into()),
                Some(btc_locked_pkscript_precompile.into()),
                load_file_as_bytes("btc_get_locked_pkscript_call_tx_data")?,
            ),
            Some("latest".to_string()),
        )
        .await?;

    assert_eq!(response, load_file_as_string("btc_get_locked_pkscript_call_response")?);

    server.stop()?;

    Ok(())
}

#[tokio::test]
async fn test_btc_last_sat_loc() -> Result<(), Box<dyn Error>> {
    if is_in_ci() {
        return Ok(());
    }

    let (server, client) = spawn_test_server().await;

    let mut btc_last_sat_loc_precompile = [0; 20];
    btc_last_sat_loc_precompile[19] = 0xfc;

    // Mine some blocks to ensure the transaction is included in a block
    client.mine(250000, 0).await?;

    let response = client
        .call(
            EthCall::new(
                Some([1u8; 20].into()),
                Some(btc_last_sat_loc_precompile.into()),
                load_file_as_bytes("btc_last_sat_loc_call_tx_data")?,
            ),
            Some("latest".to_string()),
        )
        .await?;

    assert_eq!(response, load_file_as_string("btc_last_sat_loc_call_response")?);

    server.stop()?;
    Ok(())
}

#[tokio::test]
async fn text_btc_get_tx_details() -> Result<(), Box<dyn Error>> {
    if is_in_ci() {
        return Ok(());
    }

    let (server, client) = spawn_test_server().await;

    let mut btc_get_tx_details_precompile = [0; 20];
    btc_get_tx_details_precompile[19] = 0xfd;

    // Mine some blocks to ensure the transaction is included in a block
    client.mine(250000, 0).await?;

    let response = client
        .call(
            EthCall::new(
                Some([1u8; 20].into()),
                Some(btc_get_tx_details_precompile.into()),
                load_file_as_bytes("btc_get_tx_details_call_tx_data")?,
            ),
            Some("latest".to_string()),
        )
        .await?;

    assert_eq!(response, load_file_as_string("btc_get_tx_details_call_response")?);

    server.stop()?;
    Ok(())
}
