use base64::prelude::BASE64_URL_SAFE;
use base64::Engine;
use bitcoin::{KnownHrp, Network};
use ureq::Agent;

lazy_static::lazy_static! {
    static ref BTC_CLIENT: Agent = Agent::new_with_defaults();
    static ref BITCOIN_RPC_URL: String = std::env::var("BITCOIN_RPC_URL")
            .unwrap_or("http://localhost:48332".to_string());
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
    false
}

pub fn check_bitcoin_rpc_status() -> bool {
    let response = BTC_CLIENT
        .post(&*BITCOIN_RPC_URL)
        .header(
            "Authorization",
            get_basic_auth_header(&*BITCOIN_RPC_USER, &*BITCOIN_RPC_PASSWORD),
        )
        .content_type("application/json")
        .config()
        .http_status_as_error(false)
        .build()
        .send(
            r#"{"jsonrpc": "1.0", "id": "curltest", "method": "getblockchaininfo", "params":[]}"#,
        );

    return !response.is_err() && response.unwrap().status() == 200;
}

pub fn get_raw_transaction(txid: &str) -> serde_json::Value {
    get_raw_transaction_with_retry(txid, 5)
}

pub fn get_raw_transaction_with_retry(txid: &str, retries_left: u32) -> serde_json::Value {
    // Check if the txid is a valid hex string
    if txid.chars().any(|c| !c.is_ascii_hexdigit()) {
        return serde_json::Value::Null;
    }

    let response = BTC_CLIENT
        .post(&*BITCOIN_RPC_URL)
        .header(
            "Authorization",
            get_basic_auth_header(&*BITCOIN_RPC_USER, &*BITCOIN_RPC_PASSWORD),
        )
        .content_type("application/json")
        .config()
        .http_status_as_error(false)
        .build()
        .send(
            format!(
                "{{
                \"jsonrpc\": \"1.0\",
                \"id\": \"brc20prog\",
                \"method\": \"getrawtransaction\",
                \"params\": {{\"txid\":\"{}\", \"verbose\": true}}
                }}",
                txid
            )
            .to_string(),
        );

    if response.is_err() {
        // wait and retry
        if retries_left > 0 {
            std::thread::sleep(std::time::Duration::from_secs(1));
            return get_raw_transaction_with_retry(txid, retries_left - 1);
        }
        panic!("Failed to get raw transaction. Response: {:?}", response);
    }

    let mut response = response.unwrap();

    if response.status() != 200 {
        return serde_json::Value::Null;
    }

    let body = response.body_mut().read_to_string();

    if body.is_err() {
        panic!("Failed to get raw transaction. Response: {:?}", response);
    }

    let body = body.unwrap();

    let json_result = body.parse();

    if json_result.is_err() {
        panic!("Failed to get raw transaction. Response: {:?}", response);
    }

    json_result.unwrap()
}

pub fn get_block_height(hash: &str) -> serde_json::Value {
    // Check if the hash is a valid hex string
    if hash.chars().any(|c| !c.is_ascii_hexdigit()) {
        return serde_json::Value::Null;
    }

    let response = BTC_CLIENT
        .post(&*BITCOIN_RPC_URL)
        .header(
            "Authorization",
            get_basic_auth_header(&*BITCOIN_RPC_USER, &*BITCOIN_RPC_PASSWORD),
        )
        .content_type("application/json")
        .config()
        .http_status_as_error(false)
        .build()
        .send(
            format!(
                "{{
                \"jsonrpc\": \"1.0\",
                \"id\": \"brc20prog\",
                \"method\": \"getblock\",
                \"params\": {{\"blockhash\":\"{}\", \"verbose\": true}}
                }}",
                hash
            )
            .to_string(),
        );

    if response.is_err() {
        panic!("Failed to get block height. Response: {:?}", response);
    }

    let mut response = response.unwrap();

    if response.status() != 200 {
        return serde_json::Value::Null;
    }

    response
        .body_mut()
        .read_to_string()
        .unwrap()
        .parse()
        .unwrap()
}

fn get_basic_auth_header(user: &str, pass: &str) -> String {
    let usrpw = String::from(user) + ":" + pass;
    String::from("Basic ") + &BASE64_URL_SAFE.encode(usrpw.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_raw_transaction() {
        if skip_btc_tests() {
            return;
        }

        let txid = "4183fb733b9553ca8b93208c91dda18bee3d0b8510720b15d76d979af7fd9926";
        let response = get_raw_transaction(txid);
        assert_eq!(response["result"]["txid"].as_str().unwrap(), txid);
    }
}
