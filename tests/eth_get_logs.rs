use std::error::Error;
use std::str::FromStr;

use alloy::primitives::Address;
use brc20_prog::types::{AddressED, GetLogsFilter};
use brc20_prog::Brc20ProgApiClient;
use revm::primitives::U256;
use test_utils::spawn_test_server;

// Canonical BRC20_Controller address; mint emits Transfer logs from here.
const CONTROLLER_ADDRESS: &str = "0xc54dd4581af2dbf18e4d90840226756e9d2b3cdb";

fn filter(address: Option<&str>, from: Option<&str>, to: Option<&str>) -> GetLogsFilter {
    GetLogsFilter {
        from_block: from.map(|s| s.to_string()),
        to_block: to.map(|s| s.to_string()),
        address: address.map(|a| AddressED::from(Address::from_str(a).unwrap())),
        topics: None,
    }
}

/// A deposit mints tokens via the controller, which emits a Transfer log.
/// Verifies eth_getLogs filtering by address and by block range.
///
/// Topic filtering is not exercised here — building a topic filter needs
/// serde_either::SingleOrVec, which isn't on the public API. Add a
/// dev-dependency and a topic case if topic filtering ever needs coverage.
#[tokio::test]
async fn test_eth_get_logs_filtering() -> Result<(), Box<dyn Error>> {
    let (server, client) = spawn_test_server(Default::default()).await;
    let pkscript = "7465737420706b736372697074".to_string(); // "test pkscript"
    let ticker = "TEST".to_string();
    let timestamp = 42;

    // Deploy controller (initialise also pings the unreachable Bitcoin RPC; the
    // controller is deployed before that check, so the error is expected).
    let _ = client.brc20_initialise([1u8; 32].into(), timestamp, 0).await;

    // Deposit -> mint -> Transfer log, in block 1.
    client
        .brc20_deposit(
            pkscript,
            ticker,
            U256::from(1000).into(),
            timestamp,
            [2u8; 32].into(),
            0,
            "deposit_inscription".to_string(),
        )
        .await?;
    client
        .brc20_finalise_block(timestamp, [2u8; 32].into(), 1)
        .await?;

    // Logs from the controller address are returned.
    let logs = client
        .eth_get_logs(filter(Some(CONTROLLER_ADDRESS), None, None))
        .await?;
    assert!(!logs.is_empty(), "controller should have emitted logs");

    // A different address yields nothing.
    let none = client
        .eth_get_logs(filter(
            Some("0x000000000000000000000000000000000000dead"),
            None,
            None,
        ))
        .await?;
    assert!(none.is_empty(), "unrelated address must match no logs");

    // A block range covering block 1 returns the logs (ranges are capped at 5
    // blocks, so keep them small).
    let in_range = client
        .eth_get_logs(filter(Some(CONTROLLER_ADDRESS), Some("0x0"), Some("0x1")))
        .await?;
    assert!(!in_range.is_empty(), "range covering block 1 must match");

    // A block range past block 1 excludes them.
    let out_of_range = client
        .eth_get_logs(filter(Some(CONTROLLER_ADDRESS), Some("0x2"), Some("0x4")))
        .await?;
    assert!(out_of_range.is_empty(), "range past block 1 must match none");

    server.stop()?;
    Ok(())
}
