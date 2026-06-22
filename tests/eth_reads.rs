use std::error::Error;

use alloy::primitives::U64;
use brc20_prog::types::RawBytes;
use brc20_prog::Brc20ProgApiClient;
use revm::primitives::U256;
use test_utils::{load_file_as_string, spawn_test_server};

/// Deploys a contract, finalises its block, then cross-checks the eth_* read
/// methods (block/tx getters, counts, code, storage) against known values.
#[tokio::test]
async fn test_eth_read_methods() -> Result<(), Box<dyn Error>> {
    let (server, client) = spawn_test_server(Default::default()).await;
    let from_pkscript = "7465737420706b736372697074".to_string(); // "test pkscript"
    let timestamp = 42;
    let block_hash = [0u8; 32].into();

    let deploy_data = load_file_as_string("brc20_prog_helper_deploy_tx_data")?;
    let deploy_data_length = deploy_data.len() as u64;
    let deploy = client
        .brc20_deploy(
            from_pkscript,
            RawBytes::new(deploy_data).into(),
            None,
            timestamp,
            block_hash,
            0,
            "deploy_inscription".to_string(),
            deploy_data_length,
            [1; 32].into(),
        )
        .await?;
    let contract_address = deploy.contract_address.clone().unwrap();
    let tx_hash = deploy.transaction_hash.clone();

    // Finalise block 0 so it can be queried by number and hash.
    client.brc20_finalise_block(timestamp, block_hash, 1).await?;

    // Latest block is 0.
    assert_eq!(client.eth_block_number().await?, "0x0");

    // The block by number, and the same block fetched by its hash.
    let block = client
        .eth_get_block_by_number("0x0".to_string(), Some(false))
        .await?;
    assert_eq!(block.number.uint, U64::from(0u64));
    let block_by_hash = client
        .eth_get_block_by_hash(block.hash.clone(), Some(false))
        .await?;
    assert_eq!(block_by_hash.hash, block.hash);

    // One transaction in the block, by both lookups.
    assert_eq!(
        client
            .eth_get_block_transaction_count_by_number("0x0".to_string())
            .await?,
        "0x1"
    );
    assert_eq!(
        client
            .eth_get_block_transaction_count_by_hash(block.hash.clone())
            .await?,
        "0x1"
    );

    // The deploy tx is reachable by hash and by (block, index) in both forms.
    let by_hash = client
        .eth_get_transaction_by_hash(tx_hash.clone())
        .await?
        .unwrap();
    assert_eq!(by_hash.hash, tx_hash);
    let by_number_index = client
        .eth_get_transaction_by_block_number_and_index(0, Some(0))
        .await?
        .unwrap();
    assert_eq!(by_number_index.hash, tx_hash);
    let by_hash_index = client
        .eth_get_transaction_by_block_hash_and_index(block.hash.clone(), Some(0))
        .await?
        .unwrap();
    assert_eq!(by_hash_index.hash, tx_hash);

    // Deployer nonce advanced to 1 after the deploy.
    assert_eq!(
        client
            .eth_get_transaction_count(deploy.from.clone(), "0x0".to_string())
            .await?,
        "0x1"
    );

    // The deployed contract has code and queryable storage.
    let code = client.eth_get_code(contract_address.clone()).await?;
    assert!(!code.bytecode.original_bytes().is_empty());
    let storage = client
        .eth_get_storage_at(contract_address, U256::ZERO.into())
        .await?;
    assert!(storage.starts_with("0x"));

    server.stop()?;
    Ok(())
}
