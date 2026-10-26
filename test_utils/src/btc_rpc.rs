//! Checks for the precompiles that read from a Bitcoin RPC node. Each network's test lives in
//! its own test binary: the RPC settings come from `.env.mainnet` / `.env.signet` and are
//! loaded into process-wide configuration, so two networks must never share a process.

use std::error::Error;

use brc20_prog::types::EthCall;
use brc20_prog::Brc20ProgApiClient;

use crate::{is_in_ci, load_file_as_eth_bytes, load_file_as_string, spawn_test_server};

/// Calls the getTxDetails precompile (0xfd) and checks its output and estimated gas.
pub async fn verify_btc_get_tx_details(
    envfile: &str,
    call_file: &str,
    response_file: &str,
    gas: &str,
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

    let call = EthCall::new(
        Some([1u8; 20].into()),
        Some(btc_get_tx_details_precompile.into()),
        load_file_as_eth_bytes(call_file)?,
    );

    let response = client
        .eth_call(call.clone(), Some("latest".to_string()))
        .await?;
    assert_eq!(response, load_file_as_string(response_file)?);

    let gas_response = client
        .eth_estimate_gas(call, Some("latest".to_string()))
        .await?;
    assert_eq!(gas_response.as_str().to_lowercase(), gas);

    server.stop()?;
    Ok(())
}

/// Calls the last sat location precompile (0xfc) on signet and checks its output and estimated gas.
pub async fn verify_btc_last_sat_loc_signet() -> Result<(), Box<dyn Error>> {
    if is_in_ci() {
        return Ok(());
    }

    dotenvy::from_filename_override(".env.signet").ok();

    let (server, client) = spawn_test_server(Default::default()).await;

    let mut btc_last_sat_loc_precompile = [0; 20];
    btc_last_sat_loc_precompile[19] = 0xfc;

    // Mine some blocks to ensure the transaction is included in a block
    client.brc20_mine(250000, 0).await?;

    let call = EthCall::new(
        Some([1u8; 20].into()),
        Some(btc_last_sat_loc_precompile.into()),
        load_file_as_eth_bytes("btc_last_sat_loc_signet_call_tx_data")?,
    );

    let response = client
        .eth_call(call.clone(), Some("latest".to_string()))
        .await?;
    assert_eq!(
        response,
        load_file_as_string("btc_last_sat_loc_signet_call_response")?
    );

    let gas_response = client
        .eth_estimate_gas(call, Some("latest".to_string()))
        .await?;
    assert_eq!(gas_response.as_str().to_lowercase(), "0xc8b36");

    server.stop()?;
    Ok(())
}
