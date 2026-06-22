use std::error::Error;

use brc20_prog::Brc20ProgApiClient;
use revm::primitives::U256;
use test_utils::spawn_test_server;

fn balance_to_u256(balance: &str) -> U256 {
    U256::from_str_radix(balance.trim_start_matches("0x"), 16).unwrap()
}

/// Exercises the BRC20 token controller end to end: deposit credits a
/// balance, and withdraw debits it.
#[tokio::test]
async fn test_deposit_balance_withdraw_roundtrip() -> Result<(), Box<dyn Error>> {
    let (server, client) = spawn_test_server(Default::default()).await;
    let pkscript = "7465737420706b736372697074".to_string(); // "test pkscript"
    let ticker = "TEST".to_string();
    let timestamp = 42;

    // Deploy the BRC20_Controller. brc20_initialise also pings the Bitcoin RPC,
    // which is unreachable in tests; the controller is deployed and genesis is
    // finalised before that check runs, so the resulting error is expected.
    let _ = client.brc20_initialise([1u8; 32].into(), timestamp, 0).await;

    // No deposits yet.
    let balance = client.brc20_balance(pkscript.clone(), ticker.clone()).await?;
    assert_eq!(balance_to_u256(&balance), U256::ZERO);

    // Deposit 1000, then finalise the block holding it.
    let deposit_amount = U256::from(1000);
    let receipt = client
        .brc20_deposit(
            pkscript.clone(),
            ticker.clone(),
            deposit_amount.into(),
            timestamp,
            [2u8; 32].into(),
            0,
            "deposit_inscription".to_string(),
        )
        .await?;
    assert!(!receipt.status.is_zero(), "deposit must succeed");
    client
        .brc20_finalise_block(timestamp, [2u8; 32].into(), 1)
        .await?;

    let balance = client.brc20_balance(pkscript.clone(), ticker.clone()).await?;
    assert_eq!(balance_to_u256(&balance), deposit_amount);

    // Withdraw 400, leaving 600.
    let withdraw_amount = U256::from(400);
    let receipt = client
        .brc20_withdraw(
            pkscript.clone(),
            ticker.clone(),
            withdraw_amount.into(),
            timestamp,
            [3u8; 32].into(),
            0,
            "withdraw_inscription".to_string(),
        )
        .await?;
    assert!(!receipt.status.is_zero(), "withdraw must succeed");
    client
        .brc20_finalise_block(timestamp, [3u8; 32].into(), 1)
        .await?;

    let balance = client.brc20_balance(pkscript.clone(), ticker.clone()).await?;
    assert_eq!(balance_to_u256(&balance), U256::from(600));

    server.stop()?;
    Ok(())
}
