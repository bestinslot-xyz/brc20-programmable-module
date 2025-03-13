lazy_static::lazy_static! {
    static ref BTC_CLIENT: reqwest::blocking::Client = reqwest::blocking::Client::new();
    static ref BITCOIN_RPC_URL: String = std::env::var("BITCOIN_RPC_URL")
            .unwrap_or("http://localhost:48332".to_string());
    static ref BITCOIN_RPC_USER: String = std::env::var("BITCOIN_RPC_USER")
            .unwrap_or("user".to_string());
    static ref BITCOIN_RPC_PASSWORD: String = std::env::var("BITCOIN_RPC_PASSWORD")
            .unwrap_or("password".to_string());
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
