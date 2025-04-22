use std::error::Error;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

use alloy_primitives::B256;
use bitcoin::{BlockHash, KnownHrp, Network, Txid};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use bitcoincore_rpc_json::{GetBlockResult, GetRawTransactionResult};

use crate::config::BRC20_PROG_CONFIG;

lazy_static::lazy_static! {
    static ref BTC_CLIENT: Client = {
        let auth = Auth::UserPass(
            BRC20_PROG_CONFIG.bitcoin_rpc_user.clone(),
            BRC20_PROG_CONFIG.bitcoin_rpc_password.clone(),
        );
        Client::new(&BRC20_PROG_CONFIG.bitcoin_rpc_url, auth).expect("Failed to create Bitcoin RPC client")
    };
    pub static ref BITCOIN_NETWORK : Network = {
        match BRC20_PROG_CONFIG.bitcoin_rpc_network.as_str() {
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

pub fn validate_bitcoin_rpc_status() -> Result<(), Box<dyn Error>> {
    if std::env::var("BITCOIN_RPC_URL").is_err() {
        return Err("Please set the BITCOIN_RPC_URL environment variable".into());
    }
    if std::env::var("BITCOIN_RPC_USER").is_err() {
        return Err("Please set the BITCOIN_RPC_USER environment variable".into());
    }
    if std::env::var("BITCOIN_RPC_PASSWORD").is_err() {
        return Err("Please set the BITCOIN_RPC_PASSWORD environment variable".into());
    }
    if std::env::var("BITCOIN_RPC_NETWORK").is_err() {
        return Err("Please set the BITCOIN_RPC_NETWORK environment variable".into());
    }

    let info = BTC_CLIENT.get_blockchain_info();

    let Ok(info) = info else {
        return Err("Bitcoin RPC unreachable.".into());
    };

    if info.chain != *BITCOIN_NETWORK {
        return Err(format!(
            "Bitcoin RPC network mismatch. Expected: {:?}, got: {:?}",
            *BITCOIN_NETWORK, info.chain
        )
        .into());
    }

    Ok(())
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
        if validate_bitcoin_rpc_status().is_err() {
            return;
        }

        let txid_string = "4183fb733b9553ca8b93208c91dda18bee3d0b8510720b15d76d979af7fd9926";
        let response = get_raw_transaction(&FixedBytes::from_hex(txid_string).unwrap());
        assert_eq!(response.unwrap().txid.to_string(), txid_string);
    }
}
