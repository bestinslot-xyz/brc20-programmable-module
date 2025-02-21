use revm::primitives::{Address, Bytes, U256};
use rust_embed::Embed;
use solabi::{selector, FunctionEncoder};

use crate::{types::TxInfo, INDEXER_ADDRESS};

static BRC20_CONTROLLER_PATH: &str = "BRC20_Controller";
static BRC20_CONTROLLER_ADDRESS: &str = "0xc54dd4581af2dbf18e4d90840226756e9d2b3cdb";

#[derive(Embed)]
#[folder = "contracts/"]
struct ContractAssets;

const MINT: FunctionEncoder<(String, solabi::Address, solabi::U256), (bool,)> =
    FunctionEncoder::new(selector!("mint(string,address,uint256)"));

pub fn load_brc20_mint_tx(ticker: String, address: Address, amount: U256) -> TxInfo {
    let to = address.as_slice().try_into().unwrap();

    let amount: [u8; 32] = amount.to_be_bytes();
    let amount1 = u128::from_be_bytes(amount[0..16].try_into().unwrap());
    let amount2 = u128::from_be_bytes(amount[16..32].try_into().unwrap());

    TxInfo {
        from: INDEXER_ADDRESS.parse().unwrap(),
        to: BRC20_CONTROLLER_ADDRESS.parse().ok(),
        data: Bytes::from(MINT.encode_params(&(
            ticker,
            solabi::Address(to),
            solabi::U256([amount1, amount2]),
        ))),
    }
}

const BURN: FunctionEncoder<(String, solabi::Address, solabi::U256), (bool,)> =
    FunctionEncoder::new(selector!("burn(string,address,uint256)"));

pub fn load_brc20_burn_tx(ticker: String, address: Address, amount: U256) -> TxInfo {
    let from = address.as_slice().try_into().unwrap();

    let amount: [u8; 32] = amount.to_be_bytes();
    let amount1 = u128::from_be_bytes(amount[0..16].try_into().unwrap());
    let amount2 = u128::from_be_bytes(amount[16..32].try_into().unwrap());

    TxInfo {
        from: INDEXER_ADDRESS.parse().unwrap(),
        to: BRC20_CONTROLLER_ADDRESS.parse().ok(),
        data: Bytes::from(BURN.encode_params(&(
            ticker,
            solabi::Address(from),
            solabi::U256([amount1, amount2]),
        ))),
    }
}

const BALANCE_OF: FunctionEncoder<(String, solabi::Address), (solabi::U256,)> =
    FunctionEncoder::new(selector!("balanceOf(string,address)"));

pub fn load_brc20_balance_tx(ticker: String, address: Address) -> TxInfo {
    let address = address.as_slice().try_into().unwrap();

    TxInfo {
        from: INDEXER_ADDRESS.parse().unwrap(),
        to: BRC20_CONTROLLER_ADDRESS.parse().ok(),
        data: Bytes::from(BALANCE_OF.encode_params(&(ticker, solabi::Address(address)))),
    }
}

pub fn decode_brc20_balance_result(data: Option<&Bytes>) -> U256 {
    if data.is_none() {
        return U256::ZERO;
    }
    let (result,) = BALANCE_OF.decode_returns(data.as_ref().unwrap()).unwrap();
    println!("result: {:?}", result);
    let result = result.to_be_bytes();
    U256::from_be_bytes(result)
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
