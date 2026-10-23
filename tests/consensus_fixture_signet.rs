//! Pins gas, traces, receipts, logs and trace hashes on signet: CANCUN blocks, the PRAGUE
//! activation at height 275,000, PRAGUE blocks, and RLP transaction hashes.
//! A snapshot diff here means a consensus change; see test_utils/src/consensus_fixture.rs.

use test_utils::{run_consensus_fixture, Era, FixtureSpec, CHAIN_ID_TESTNETS};

#[tokio::test]
async fn consensus_fixture_signet() {
    let report = run_consensus_fixture(FixtureSpec {
        network: "signet",
        chain_id: CHAIN_ID_TESTNETS,
        eras: vec![
            Era {
                name: "signet cancun",
                start_height: None,
                transact_file: None,
            },
            Era {
                name: "signet prague",
                start_height: Some(275_000),
                transact_file: Some("consensus_fixture/fixture_txs_testnets"),
            },
        ],
    })
    .await;
    insta::assert_snapshot!(report);
}
