//! Bitcoin RPC precompiles against the mainnet node configured in `.env.mainnet`.
//! Skipped in CI. Kept in its own binary; see test_utils/src/btc_rpc.rs.

use std::error::Error;

use test_utils::{is_in_ci, verify_btc_get_tx_details};

#[tokio::test]
async fn test_btc_rpc_precompiles_mainnet() -> Result<(), Box<dyn Error>> {
    if is_in_ci() {
        return Ok(());
    }

    verify_btc_get_tx_details(
        ".env.mainnet",
        "btc_get_tx_details_mainnet_call_1_tx_data",
        "btc_get_tx_details_mainnet_call_1_response",
        "0xc8b36",
    )
    .await
    .expect("Failed to verify btc_get_tx_details mainnet call 1");
    verify_btc_get_tx_details(
        ".env.mainnet",
        "btc_get_tx_details_mainnet_call_2_tx_data",
        "btc_get_tx_details_mainnet_call_2_response",
        "0x18c463",
    )
    .await
    .expect("Failed to verify btc_get_tx_details mainnet call 2");
    verify_btc_get_tx_details(
        ".env.mainnet",
        "btc_get_tx_details_mainnet_call_3_tx_data",
        "btc_get_tx_details_mainnet_call_3_response",
        "0x24fd91",
    )
    .await
    .expect("Failed to verify btc_get_tx_details mainnet call 3");

    Ok(())
}
