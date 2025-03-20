use bitcoin::{KnownHrp, Network};

lazy_static::lazy_static! {
    static ref BTC_CLIENT: reqwest::blocking::Client = reqwest::blocking::Client::new();
    static ref BITCOIN_RPC_URL: String = std::env::var("BITCOIN_RPC_URL")
            .unwrap_or("http://localhost:48332".to_string());
    static ref BITCOIN_RPC_USER: String = std::env::var("BITCOIN_RPC_USER")
            .unwrap_or("user".to_string());
    static ref BITCOIN_RPC_PASSWORD: String = std::env::var("BITCOIN_RPC_PASSWORD")
            .unwrap_or("password".to_string());
    static ref BITCOIN_NETWORK_STRING: String = std::env::var("BITCOIN_NETWORK")
            .unwrap_or("testnet4".to_string());
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

pub fn get_raw_transaction(txid: &str) -> serde_json::Value {
    let response = BTC_CLIENT
        .post(&*BITCOIN_RPC_URL)
        .basic_auth(&*BITCOIN_RPC_USER, Some(&*BITCOIN_RPC_PASSWORD))
        .body(
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
        )
        .send()
        .unwrap();

    response.json().unwrap()
}
