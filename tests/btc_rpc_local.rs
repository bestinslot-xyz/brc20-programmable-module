use std::env;
use std::error::Error;

use brc20_prog::types::EthCall;
use brc20_prog::Brc20ProgApiClient;
use test_utils::{load_file_as_eth_bytes, load_file_as_string, spawn_test_server};

/// Local-only end-to-end check of the getTxDetails Bitcoin-RPC precompile
/// against a live signet node. There is no mocking — it talks to a real
/// Bitcoin RPC, so it is opt-in.
///
/// To run it:
///   1. Provide signet RPC settings, e.g. a `.env.signet` file with
///      BITCOIN_RPC_URL / BITCOIN_RPC_USER / BITCOIN_RPC_PASSWORD /
///      BITCOIN_RPC_NETWORK=signet.
///   2. `BRC20_RUN_BTC_RPC_TESTS=1 cargo test --all-features --test btc_rpc_local`
///
/// Without the opt-in flag it skips, so it is safe to leave in the suite and in
/// CI (where no Bitcoin node is reachable).
#[tokio::test]
async fn test_btc_get_tx_details_signet_live() -> Result<(), Box<dyn Error>> {
    if env::var("BRC20_RUN_BTC_RPC_TESTS").ok().as_deref() != Some("1") {
        eprintln!(
            "skipping live BTC RPC test; set BRC20_RUN_BTC_RPC_TESTS=1 (and signet RPC env) to run"
        );
        return Ok(());
    }

    // Load signet RPC settings if a .env.signet file is present.
    dotenvy::from_filename_override(".env.signet").ok();

    let (server, client) = spawn_test_server(Default::default()).await;

    let mut get_tx_details_precompile = [0u8; 20];
    get_tx_details_precompile[19] = 0xfd;

    // Mine enough blocks for the referenced transaction to be treated as final.
    client.brc20_mine(300000, 0).await?;

    let response = client
        .eth_call(
            EthCall::new(
                Some([1u8; 20].into()),
                Some(get_tx_details_precompile.into()),
                load_file_as_eth_bytes("btc_get_tx_details_signet_call_tx_data")?,
            ),
            Some("latest".to_string()),
        )
        .await?;

    assert_eq!(
        response,
        load_file_as_string("btc_get_tx_details_signet_call_response")?
    );

    server.stop()?;
    Ok(())
}
