use std::str::FromStr;

use alloy::primitives::{Address, Bytes, U256};
use alloy_sol_types::{sol, SolCall};
use revm::primitives::TxKind;
use rust_embed::Embed;

use crate::engine::TxInfo;
use crate::global::INDEXER_ADDRESS;

lazy_static::lazy_static! {
    pub static ref BRC20_CONTROLLER_PATH: String = "BRC20_Controller".to_string();
    pub static ref BRC20_CONTROLLER_ADDRESS: Address = "0xc54dd4581af2dbf18e4d90840226756e9d2b3cdb".parse().unwrap();
}

#[derive(Embed)]
#[folder = "src/brc20_controller/contract/output"]
struct ContractAssets;

sol! {
    function mint(bytes, address, uint256) returns (bool);
    function burn(bytes, address, uint256) returns (bool);
    function balanceOf(bytes, address) returns (uint256);
}

pub fn load_brc20_mint_tx(ticker: Bytes, address: Address, amount: U256) -> TxInfo {
    TxInfo::from_inscription(
        *INDEXER_ADDRESS,
        TxKind::Call(*BRC20_CONTROLLER_ADDRESS),
        mintCall::new((ticker, address, amount)).abi_encode().into(),
    )
}

pub fn load_brc20_burn_tx(ticker: Bytes, address: Address, amount: U256) -> TxInfo {
    TxInfo::from_inscription(
        *INDEXER_ADDRESS,
        TxKind::Call(*BRC20_CONTROLLER_ADDRESS),
        burnCall::new((ticker, address, amount)).abi_encode().into(),
    )
}

pub fn load_brc20_balance_tx(ticker: Bytes, address: Address) -> TxInfo {
    TxInfo::from_inscription(
        *INDEXER_ADDRESS,
        TxKind::Call(*BRC20_CONTROLLER_ADDRESS),
        balanceOfCall::new((ticker, address)).abi_encode().into(),
    )
}

pub fn decode_brc20_balance_result(data: Option<&Bytes>) -> U256 {
    let Some(data) = data else {
        return U256::ZERO;
    };
    let returns = balanceOfCall::abi_decode_returns(data);
    return returns.ok().unwrap_or(U256::ZERO);
}

pub fn load_brc20_deploy_tx() -> TxInfo {
    let file_content = ContractAssets::get(&format!("{}_deploy.bytecode", *BRC20_CONTROLLER_PATH))
        .expect("Failed to load contract binary");
    let data = String::from_utf8(file_content.data.to_vec())
        .expect("Failed to convert binary data to string");

    TxInfo::from_inscription(
        *INDEXER_ADDRESS,
        TxKind::Create,
        Bytes::from_str(&data).expect("Failed to convert string to bytes"),
    )
}

pub fn verify_brc20_contract_address(address: &str) -> Result<(), String> {
    let address: Address = address.parse().unwrap_or_default();
    if address == *BRC20_CONTROLLER_ADDRESS {
        return Ok(());
    }
    Err("Invalid BRC20_Controller contract address".to_string())
}
