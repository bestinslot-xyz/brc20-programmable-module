use std::error::Error;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

use alloy_primitives::B256;
use bitcoin::{BlockHash, KnownHrp, Network, Txid};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use bitcoincore_rpc_json::{GetBlockResult, GetRawTransactionResult};

lazy_static::lazy_static! {
    static ref BTC_CLIENT: Client = {
        let auth = Auth::UserPass(
            BITCOIN_RPC_USER.to_string(),
            BITCOIN_RPC_PASSWORD.to_string(),
        );
        Client::new(&*BITCOIN_RPC_URL, auth).expect("Failed to create Bitcoin RPC client")
    };
    static ref BITCOIN_RPC_URL: String = std::env::var("BITCOIN_RPC_URL")
            .unwrap_or("http://localhost:38332".to_string());
    static ref BITCOIN_RPC_USER: String = std::env::var("BITCOIN_RPC_USER")
            .unwrap_or("user".to_string());
    static ref BITCOIN_RPC_PASSWORD: String = std::env::var("BITCOIN_RPC_PASSWORD")
            .unwrap_or("password".to_string());
    static ref BITCOIN_NETWORK_STRING: String = std::env::var("BITCOIN_NETWORK")
            .unwrap_or("signet".to_string());
    pub static ref BITCOIN_NETWORK : Network = {
        match BITCOIN_NETWORK_STRING.as_str() {
            "mainnet" => Network::Bitcoin,
            "signet" => Network::Signet,
            "testnet" => Network::Testnet,
            "testnet4" => Network::Testnet4,
            "regtest" => Network::Regtest,
            _ => Network::Testnet4,
        }
    };
    pub static ref BITCOIN_HRP: KnownHrp = {
        match *BITCOIN_NETWORK {
            Network::Bitcoin => KnownHrp::Mainnet,
            Network::Testnet => KnownHrp::Testnets,
            Network::Testnet4 => KnownHrp::Testnets,
            Network::Signet => KnownHrp::Testnets,
            Network::Regtest => KnownHrp::Regtest,
            _ => KnownHrp::Testnets,
        }
    };
}

#[cfg(test)]
pub fn skip_btc_tests() -> bool {
    if !check_bitcoin_rpc_status() {
        if std::env::var("BITCOIN_RPC_URL").is_err() {
            println!("Please set the BITCOIN_RPC_URL environment variable");
            return true;
        }
        if std::env::var("BITCOIN_RPC_USER").is_err() {
            println!("Please set the BITCOIN_RPC_USER environment variable");
            return true;
        }
        if std::env::var("BITCOIN_RPC_PASSWORD").is_err() {
            println!("Please set the BITCOIN_RPC_PASSWORD environment variable");
            return true;
        }
        if std::env::var("BITCOIN_NETWORK").is_err() {
            println!("Please set the BITCOIN_NETWORK environment variable");
            return true;
        }
        println!("Bitcoin RPC is unreachable.");
        return true;
    }
    false
}

pub fn check_bitcoin_rpc_status() -> bool {
    return !BTC_CLIENT.get_blockchain_info().is_err();
}

pub fn get_raw_transaction(txid: &B256) -> Result<GetRawTransactionResult, Box<dyn Error>> {
    let bitcoin_txid = Txid::from_str(&hex::encode(txid.as_slice()).to_lowercase().as_str())
        .map_err(|_| "Invalid Txid")?;
    get_raw_transaction_with_retry(&bitcoin_txid, 5)
}

fn get_raw_transaction_with_retry(
    txid: &Txid,
    retries_left: u32,
) -> Result<GetRawTransactionResult, Box<dyn Error>> {
    match BTC_CLIENT.get_raw_transaction_info(&txid, None) {
        Ok(response) => return Ok(response),
        Err(error) => {
            if retries_left > 0 {
                sleep(Duration::from_secs(1));
                return get_raw_transaction_with_retry(txid, retries_left - 1);
            }
            panic!("Bitcoin RPC unreachable. Response: {:?}", error);
        }
    };
}

pub fn get_block_info(block_hash: &BlockHash) -> Result<GetBlockResult, Box<dyn Error>> {
    get_block_info_with_retry(block_hash, 5)
}

fn get_block_info_with_retry(
    block_hash: &BlockHash,
    retries_left: u32,
) -> Result<GetBlockResult, Box<dyn Error>> {
    match BTC_CLIENT.get_block_info(&block_hash) {
        Ok(response) => return Ok(response),
        Err(error) => {
            if retries_left > 0 {
                sleep(Duration::from_secs(1));
                return get_block_info_with_retry(block_hash, retries_left - 1);
            }
            panic!("Bitcoin RPC unreachable. Response: {:?}", error);
        }
    };
}

#[cfg(test)]
mod tests {
    use alloy_primitives::hex::FromHex;
    use alloy_primitives::FixedBytes;

    use super::*;

    #[test]
    fn test_get_raw_transaction() {
        if skip_btc_tests() {
            return;
        }

        let txid_string = "4183fb733b9553ca8b93208c91dda18bee3d0b8510720b15d76d979af7fd9926";
        let response = get_raw_transaction(&FixedBytes::from_hex(txid_string).unwrap());
        assert_eq!(response.unwrap().txid.to_string(), txid_string);
    }
}
