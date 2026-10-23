//! Pins gas, traces, receipts, logs and trace hashes on mainnet below the PRAGUE and RLP tx
//! hash activation heights: CANCUN rules and signing-hash transaction hashes.
//! A snapshot diff here means a consensus change; see test_utils/src/consensus_fixture.rs.

use test_utils::{run_consensus_fixture, Era, FixtureSpec, CHAIN_ID_MAINNET};

#[tokio::test]
async fn consensus_fixture_mainnet() {
    let report = run_consensus_fixture(FixtureSpec {
        network: "bitcoin",
        chain_id: CHAIN_ID_MAINNET,
        eras: vec![Era {
            name: "mainnet cancun",
            start_height: None,
            transact_file: Some("consensus_fixture/fixture_txs_mainnet"),
        }],
    })
    .await;
    insta::assert_snapshot!(report);
}
