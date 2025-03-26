use alloy_primitives::U256;
use alloy_sol_types::{sol, SolCall};
use revm::primitives::{Address, Bytes};
use rust_embed::Embed;

use crate::server::types::TxInfo;
use crate::server::INDEXER_ADDRESS;

static BRC20_CONTROLLER_PATH: &str = "BRC20_Controller";
static BRC20_CONTROLLER_ADDRESS: &str = "0xc54dd4581af2dbf18e4d90840226756e9d2b3cdb";

#[derive(Embed)]
#[folder = "src/brc20_controller/contract/"]
struct ContractAssets;

sol! {
    function mint(string, address, uint256) returns (bool);
    function burn(string, address, uint256) returns (bool);
    function balanceOf(string, address) returns (uint256);
}

pub fn load_brc20_mint_tx(ticker: String, address: Address, amount: U256) -> TxInfo {
    let to = address.as_slice().try_into().unwrap();

    TxInfo {
        from: INDEXER_ADDRESS.parse().unwrap(),
        to: BRC20_CONTROLLER_ADDRESS.parse().ok(),
        data: Bytes::from(mintCall::new((ticker, Address(to), amount)).abi_encode()),
    }
}

pub fn load_brc20_burn_tx(ticker: String, address: Address, amount: U256) -> TxInfo {
    let from = address.as_slice().try_into().unwrap();

    TxInfo {
        from: INDEXER_ADDRESS.parse().unwrap(),
        to: BRC20_CONTROLLER_ADDRESS.parse().ok(),
        data: Bytes::from(burnCall::new((ticker, Address(from), amount)).abi_encode()),
    }
}

pub fn load_brc20_balance_tx(ticker: String, address: Address) -> TxInfo {
    let address = address.as_slice().try_into().unwrap();

    TxInfo {
        from: INDEXER_ADDRESS.parse().unwrap(),
        to: BRC20_CONTROLLER_ADDRESS.parse().ok(),
        data: Bytes::from(balanceOfCall::new((ticker, Address(address))).abi_encode()),
    }
}

pub fn decode_brc20_balance_result(data: Option<&Bytes>) -> U256 {
    if data.is_none() {
        return U256::ZERO;
    }
    let result = balanceOfCall::abi_decode_returns(data.unwrap(), false);
    if result.is_err() {
        return U256::ZERO;
    }
    result.unwrap()._0
}

pub fn load_brc20_deploy_tx() -> TxInfo {
    let file_content = ContractAssets::get(&format!("{}.bin", BRC20_CONTROLLER_PATH));
    let file_content = file_content.unwrap();
    let data = String::from_utf8(file_content.data.to_vec()).unwrap();

    TxInfo {
        from: INDEXER_ADDRESS.parse().unwrap(),
        to: None,
        data: Bytes::from(hex::decode(data).unwrap()),
    }
}

pub fn verify_brc20_contract_address(address: &str) {
    assert_eq!(address.to_lowercase(), BRC20_CONTROLLER_ADDRESS);
}
