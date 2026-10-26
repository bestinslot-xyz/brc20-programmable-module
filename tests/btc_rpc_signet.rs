//! Bitcoin RPC precompiles against the signet node configured in `.env.signet`.
//! Skipped in CI. Kept in its own binary; see test_utils/src/btc_rpc.rs.

use std::error::Error;

use test_utils::{is_in_ci, verify_btc_get_tx_details, verify_btc_last_sat_loc_signet};

#[tokio::test]
async fn test_btc_rpc_precompiles_signet() -> Result<(), Box<dyn Error>> {
    if is_in_ci() {
        return Ok(());
    }

    verify_btc_get_tx_details(
        ".env.signet",
        "btc_get_tx_details_signet_call_tx_data",
        "btc_get_tx_details_signet_call_response",
        "0xc8b36",
    )
    .await
    .expect("Failed to verify btc_get_tx_details signet call");
    verify_btc_last_sat_loc_signet()
        .await
        .expect("Failed to verify btc_last_sat_loc signet call");

    Ok(())
}
