use std::str::FromStr;

use db::DB;
use revm::{
    precompile::Error,
    primitives::{Bytes, PrecompileErrors, PrecompileOutput, PrecompileResult, B256},
    ContextStatefulPrecompile,
};
use solabi::{selector, FunctionEncoder, U256};

lazy_static::lazy_static! {
    static ref BTC_CLIENT: reqwest::blocking::Client = reqwest::blocking::Client::new();
    static ref BITCOIN_RPC_URL: String = std::env::var("BITCOIN_RPC_URL")
            .unwrap_or("http://localhost:48332".to_string());
    static ref BITCOIN_RPC_USER: String = std::env::var("BITCOIN_RPC_USER")
            .unwrap_or("user".to_string());
    static ref BITCOIN_RPC_PASSWORD: String = std::env::var("BITCOIN_RPC_PASSWORD")
            .unwrap_or("password".to_string());
}

pub struct BTCPrecompile;

/// Signature for the getTxDetails function in the BTCPrecompile contract
/// Uses get raw tx details from the blockchain using the json rpc and returns the details from the transaction
/// ScriptPubKey for the vin transaction is fetched using the txid and vout
///
/// # Returns (block_height, vin_txid, vin_vout, vin_scriptPubKey_hex, vin_value, vout_scriptPubKey_hex, vout_value) in a tuple
/// # Errors - Returns an error if the transaction details are not found
const TX_DETAILS: FunctionEncoder<String, (U256, String, U256, String, U256, String, U256)> =
    FunctionEncoder::new(selector!("getTxDetails(string)"));

impl ContextStatefulPrecompile<DB> for BTCPrecompile {
    fn call(
        &self,
        bytes: &Bytes,
        gas_limit: u64,
        _evmctx: &mut revm::InnerEvmContext<DB>,
    ) -> PrecompileResult {
        let result = TX_DETAILS.decode_params(&bytes);

        if result.is_err() {
            return Err(PrecompileErrors::Error(Error::Other(
                "Invalid params".to_string(),
            )));
        }

        let txid = result.unwrap();

        let response = get_raw_transaction(&txid);
        if response["error"].is_object() {
            return Err(PrecompileErrors::Error(Error::Other(
                response["error"]["message"].as_str().unwrap().to_string(),
            )));
        }

        let response = response["result"].clone();

        let block_hash = response["blockhash"].as_str().unwrap_or("").to_string();
        let block_height = _evmctx
            .db
            .get_block_number(B256::from_str(&block_hash).unwrap_or(B256::ZERO))
            .unwrap()
            .map(|x| x.0.as_limbs()[0])
            .unwrap_or(0);

        let vin = response["vin"][0].clone();
        let vin_txid = vin["txid"].as_str().unwrap_or("").to_string();
        let vin_vout = vin["vout"].as_u64().unwrap_or(0);
        let vout = response["vout"][0].clone();
        let vout_script_pub_key_hex = vout["scriptPubKey"]["hex"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let vout_value = vout["value"].as_f64().unwrap_or(0.0);
        let vout_value = (vout_value * 100000000.0) as u64;

        // Get the scriptPubKey from the vin transaction, using the txid and vout
        let vin_script_pub_key_response = get_raw_transaction(&vin_txid);
        if vin_script_pub_key_response["error"].is_object() {
            return Err(PrecompileErrors::Error(Error::Other(
                vin_script_pub_key_response["error"]["message"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            )));
        }

        let vin_script_pub_key_response = vin_script_pub_key_response["result"].clone();
        let vin_script_pub_key_hex = vin_script_pub_key_response["vout"][vin_vout as usize]
            ["scriptPubKey"]["hex"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let vin_value = vin_script_pub_key_response["vout"][vin_vout as usize]["value"]
            .as_f64()
            .unwrap_or(0.0);
        let vin_value = (vin_value * 100000000.0) as u64;

        let bytes = TX_DETAILS.encode_returns(&(
            U256::from(block_height),
            vin_txid,
            U256::from(vin_vout),
            vin_script_pub_key_hex,
            U256::from(vin_value),
            vout_script_pub_key_hex,
            U256::from(vout_value),
        ));

        let gas_used = 100000;
        if gas_used > gas_limit {
            return Err(PrecompileErrors::Error(Error::OutOfGas));
        }
        Ok(PrecompileOutput {
            bytes: Bytes::from(bytes),
            gas_used,
        })
    }
}

fn get_raw_transaction(txid: &str) -> serde_json::Value {
    let response = BTC_CLIENT
        .post(&*BITCOIN_RPC_URL)
        .basic_auth(&*BITCOIN_RPC_USER, Some(&*BITCOIN_RPC_PASSWORD))
        .body(
            format!(
                "{{
                \"jsonrpc\": \"1.0\",
                \"id\": \"b2p\",
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

#[cfg(test)]
mod tests {
    use solabi::U256;

    use super::TX_DETAILS;

    #[test]
    fn test_get_tx_details_encode_params() {
        let txid = "ab6baebc91d645aade178f952bf75e62735b37ee692717e090b1b1f2a2b243ba";
        let data = TX_DETAILS.encode_params(&txid.to_string());
        assert_eq!(
            hex::encode(data),
            "96327323000000000000000000000000000000000000000000000000000000000000004061623662616562633931643634356161646531373866393532626637356536323733356233376565363932373137653039306231623166326132623234336261"
        );
    }

    #[test]
    fn test_get_tx_details_decode_returns() {
        let data = "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000005a5b600000000000000000000000000000000000000000000000000000000000001c0000000000000000000000000000000000000000000000000000000000000014a00000000000000000000000000000000000000000000000000000000000000403630393262653136353461363939373365663034316366633932353736303939393734623762633366383035633762336162666364346337613430363238616100000000000000000000000000000000000000000000000000000000000000443531323062353339656566646438633237346161613934623738666635626263346437363834363438303330363363353735383238313438633364393134306165366236000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000044353132306663646335613762643636623464336138633931663161316366393461643764353631663361333034626631386661663536373862316565343765373833623700000000000000000000000000000000000000000000000000000000";
        let (
            block_height,
            vin_txid,
            vin_vout,
            vin_script_pub_key_hex,
            vin_value,
            vout_script_pub_key_hex,
            vout_value,
        ) = TX_DETAILS
            .decode_returns(hex::decode(data).unwrap().as_slice())
            .unwrap();

        assert_eq!(block_height, U256::from(0u64));
        assert_eq!(
            vin_txid,
            "6092be1654a69973ef041cfc92576099974b7bc3f805c7b3abfcd4c7a40628aa"
        );
        assert_eq!(vin_vout, U256::from(0u64));
        assert_eq!(
            vin_script_pub_key_hex,
            "5120b539eefdd8c274aaa94b78ff5bbc4d768464803063c575828148c3d9140ae6b6"
        );
        assert_eq!(vin_value, U256::from(370102u64));
        assert_eq!(
            vout_script_pub_key_hex,
            "5120fcdc5a7bd66b4d3a8c91f1a1cf94ad7d561f3a304bf18faf5678b1ee47e783b7"
        );
        assert_eq!(vout_value, U256::from(330u64));
    }
}
