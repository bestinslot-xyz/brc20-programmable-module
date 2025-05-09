mod common;
use std::error::Error;

use brc20_prog::{
    types::EthCall,
    Brc20ProgApiClient,
};
use common::{is_in_ci, load_file_as_bytes, spawn_test_server};

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

    assert_eq!(response, "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002251205b85902c60f0e252ddaedade40ba1f115d483305c00172a15dbccc58fcf8eb55000000000000000000000000000000000000000000000000000000000000");

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

    assert_eq!(response, "0x8d4bc3ac21211723436e35ffbf32a58f74fe942e0ea10936504db07afb1af7c30000000000000000000000000000000000000000000000000000000000000013000000000000000000000000000000000000000000000000000000000003d09000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000002251204a6041f54b8cf8b2d48c6f725cb0514e51e5e7e7ac429c33da62e98765dd62f300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000160014f477952f33561c1b89a1fe9f28682f623263e15900000000000000000000");

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

    assert_eq!(response, "0x000000000000000000000000000000000000000000000000000000000003ad4000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001600000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000024000000000000000000000000000000000000000000000000000000000000002c000000000000000000000000000000000000000000000000000000000000000018d4bc3ac21211723436e35ffbf32a58f74fe942e0ea10936504db07afb1af7c30000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000001300000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002251204a6041f54b8cf8b2d48c6f725cb0514e51e5e7e7ac429c33da62e98765dd62f3000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000009896800000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000160014f477952f33561c1b89a1fe9f28682f623263e1590000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000935e90");

    server.stop()?;
    Ok(())
}
