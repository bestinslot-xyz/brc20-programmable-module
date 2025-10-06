use std::error::Error;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

use alloy::primitives::B256;
use bitcoin::{BlockHash, KnownHrp, Network, Transaction, Txid};
use bitcoincore_rpc::jsonrpc::Error::Rpc;
use bitcoincore_rpc::Error::JsonRpc;
use bitcoincore_rpc::{Auth, Client, RpcApi};

use crate::global::{SharedData, CONFIG};

lazy_static::lazy_static! {
    static ref BITCOIN_RPC_URL: SharedData<String> = SharedData::new(Default::default());
    static ref BITCOIN_RPC_USER: SharedData<String> = SharedData::new(Default::default());
    static ref BITCOIN_RPC_PASSWORD: SharedData<String> = SharedData::new(Default::default());
    static ref BTC_CLIENT: SharedData<Client> = {
        let auth = Auth::UserPass(
            CONFIG.read().bitcoin_rpc_user.clone(),
            CONFIG.read().bitcoin_rpc_password.clone(),
        );
        SharedData::new(Client::new(&CONFIG.read().bitcoin_rpc_url, auth).expect("Failed to create Bitcoin RPC client"))
    };
}

pub fn update_bitcoin_client() {
    // if config has changed, create a new client
    if !BITCOIN_RPC_URL.read().eq(&CONFIG.read().bitcoin_rpc_url)
        || !BITCOIN_RPC_USER.read().eq(&CONFIG.read().bitcoin_rpc_user)
        || !BITCOIN_RPC_PASSWORD
            .read()
            .eq(&CONFIG.read().bitcoin_rpc_password)
    {
        BITCOIN_RPC_URL.write_fn_unchecked(|url| {
            *url = CONFIG.read().bitcoin_rpc_url.clone();
        });
        BITCOIN_RPC_USER.write_fn_unchecked(|user| {
            *user = CONFIG.read().bitcoin_rpc_user.clone();
        });
        BITCOIN_RPC_PASSWORD.write_fn_unchecked(|password| {
            *password = CONFIG.read().bitcoin_rpc_password.clone();
        });
        let auth = Auth::UserPass(
            CONFIG.read().bitcoin_rpc_user.clone(),
            CONFIG.read().bitcoin_rpc_password.clone(),
        );
        BTC_CLIENT.write_fn_unchecked(|client| {
            *client = Client::new(&CONFIG.read().bitcoin_rpc_url, auth)
                .expect("Failed to create Bitcoin RPC client");
        });
    }
}

pub fn get_bitcoin_network() -> Network {
    match CONFIG.read().bitcoin_rpc_network.as_str() {
        "bitcoin" => Network::Bitcoin,
        "mainnet" => Network::Bitcoin,
        "signet" => Network::Signet,
        "testnet" => Network::Testnet,
        "testnet4" => Network::Testnet4,
        "regtest" => Network::Regtest,
        _ => Network::Testnet4,
    }
}

pub fn get_bitcoin_hrp() -> KnownHrp {
    match get_bitcoin_network() {
        Network::Bitcoin => KnownHrp::Mainnet,
        Network::Testnet => KnownHrp::Testnets,
        Network::Testnet4 => KnownHrp::Testnets,
        Network::Signet => KnownHrp::Testnets,
        Network::Regtest => KnownHrp::Regtest,
        _ => KnownHrp::Testnets,
    }
}

pub fn validate_bitcoin_rpc_status() -> Result<(), Box<dyn Error>> {
    if CONFIG.read().bitcoin_rpc_url.is_empty() {
        return Err("Please configure BITCOIN_RPC_URL".into());
    }
    if CONFIG.read().bitcoin_rpc_network.is_empty() {
        return Err("Please configure BITCOIN_RPC_NETWORK".into());
    }

    // Update the client if the config has changed
    update_bitcoin_client();
    let info = BTC_CLIENT.read().get_blockchain_info();

    let Ok(info) = info else {
        return Err("Bitcoin RPC unreachable.".into());
    };

    if info.chain != get_bitcoin_network() {
        return Err(format!(
            "Bitcoin RPC network mismatch. Expected: {:?}, got: {:?}",
            get_bitcoin_network(), info.chain
        )
        .into());
    }

    Ok(())
}

pub fn get_transaction_and_block_hash(
    txid: &B256,
) -> Result<(Transaction, Option<BlockHash>), Box<dyn Error>> {
    let bitcoin_txid = Txid::from_str(&hex::encode(txid.as_slice()).to_lowercase().as_str())
        .map_err(|_| "Invalid Txid")?;
    get_transaction_and_block_hash_with_retry(&bitcoin_txid, 5)
}

fn get_transaction_and_block_hash_with_retry(
    txid: &Txid,
    retries_left: u32,
) -> Result<(Transaction, Option<BlockHash>), Box<dyn Error>> {
    match BTC_CLIENT.read().get_raw_transaction_info(&txid, None) {
        Ok(result) => match result.transaction() {
            Ok(tx) => Ok((tx.clone(), result.blockhash)),
            Err(_) => Err("Unable to decode transaction".into()),
        },
        Err(error) => {
            // Error code -5 is "RPC_INVALID_ADDRESS_OR_KEY", which means the txid is not found
            if let JsonRpc(Rpc(ref rpc_error)) = error {
                if rpc_error.code == -5 {
                    // Transaction not found, return error
                    return Err(format!("Tx not found. Txid: {:?}", txid).into());
                }
            }
            // Other error, retry
            if retries_left > 0 {
                sleep(Duration::from_secs(1));
                get_transaction_and_block_hash_with_retry(txid, retries_left - 1)
            } else {
                panic!("Bitcoin RPC unreachable. Response: {:?}", error);
            }
        }
    }
}

pub fn get_transaction(txid: &B256) -> Result<Transaction, Box<dyn Error>> {
    let bitcoin_txid = Txid::from_str(&hex::encode(txid.as_slice()).to_lowercase().as_str())
        .map_err(|_| "Invalid Txid")?;
    get_transaction_with_retry(&bitcoin_txid, 5)
}

fn get_transaction_with_retry(
    txid: &Txid,
    retries_left: u32,
) -> Result<Transaction, Box<dyn Error>> {
    match BTC_CLIENT.read().get_raw_transaction(&txid, None) {
        Ok(result) => Ok(result),
        Err(error) => {
            // Error code -5 is "RPC_INVALID_ADDRESS_OR_KEY", which means the txid is not found
            if let JsonRpc(Rpc(ref rpc_error)) = error {
                if rpc_error.code == -5 {
                    // Transaction not found, return error
                    return Err(format!("Tx not found. Txid: {:?}", txid).into());
                }
            }
            // Other error, retry
            if retries_left > 0 {
                sleep(Duration::from_secs(1));
                get_transaction_with_retry(txid, retries_left - 1)
            } else {
                panic!("Bitcoin RPC unreachable. Response: {:?}", error);
            }
        }
    }
}

pub fn get_block_height(block_hash: &BlockHash) -> Result<usize, Box<dyn Error>> {
    get_block_height_with_retry(block_hash, 5)
}

fn get_block_height_with_retry(
    block_hash: &BlockHash,
    retries_left: u32,
) -> Result<usize, Box<dyn Error>> {
    match BTC_CLIENT.read().get_block_header_info(&block_hash) {
        Ok(response) => return Ok(response.height),
        Err(error) => {
            if retries_left > 0 {
                sleep(Duration::from_secs(1));
                return get_block_height_with_retry(block_hash, retries_left - 1);
            }
            panic!("Bitcoin RPC unreachable. Response: {:?}", error);
        }
    };
}

#[cfg(test)]
mod tests {
    use alloy::primitives::hex::FromHex;
    use alloy::primitives::FixedBytes;

    use super::*;

    #[test]
    fn test_get_raw_transaction() {
        if validate_bitcoin_rpc_status().is_err() {
            return;
        }

        let txid_string = "4183fb733b9553ca8b93208c91dda18bee3d0b8510720b15d76d979af7fd9926";
        let response = get_transaction_and_block_hash(&FixedBytes::from_hex(txid_string).unwrap());
        assert_eq!(response.unwrap().0.compute_txid().to_string(), txid_string);
    }
}
