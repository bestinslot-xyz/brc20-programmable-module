use db::DB;
use revm::{
    precompile::Error,
    primitives::{Bytes, PrecompileErrors, PrecompileOutput, PrecompileResult},
    ContextStatefulPrecompile,
};
use solabi::{selector, FunctionEncoder, U256};

use super::btc_utils::get_raw_transaction;

static GAS_PER_RPC_CALL: u64 = 100000;

pub struct LastSatLocationPrecompile;

/// Signature for the getLastSatLocation function in the LastSatLocationPrecompile contract
/// Uses bitcoin rpc to get the transaction details for the given txid
/// Then returns the last sat location for the given vout and sat
///
/// # Returns (txid, vout, sat, old_pkscript, new_pkscript) in a tuple
/// # Errors - Returns an error if the transaction details are not found
const LAST_SAT_LOCATION: FunctionEncoder<
    (String, U256, U256),
    (String, U256, U256, String, String),
> = FunctionEncoder::new(selector!("getLastSatLocation(string, uint256, uint256)"));

impl ContextStatefulPrecompile<DB> for LastSatLocationPrecompile {
    fn call(
        &self,
        bytes: &Bytes,
        gas_limit: u64,
        _evmctx: &mut revm::InnerEvmContext<DB>,
    ) -> PrecompileResult {
        let result = LAST_SAT_LOCATION.decode_params(&bytes);

        if result.is_err() {
            return Err(PrecompileErrors::Error(Error::Other(
                "Invalid params".to_string(),
            )));
        }

        let (txid, vout, sat) = result.unwrap();

        let vout = vout.as_u64() as usize;
        let sat = sat.as_u64();

        let mut gas_used = GAS_PER_RPC_CALL;
        if gas_used > gas_limit {
            return Err(PrecompileErrors::Error(Error::OutOfGas));
        }
        let response = get_raw_transaction(&txid);

        if response["error"].is_object() {
            return Err(PrecompileErrors::Error(Error::Other(
                response["error"]["message"].as_str().unwrap().to_string(),
            )));
        }

        let response = response["result"].clone();

        if response["vin"][0]["coinbase"].is_string() {
            return Err(PrecompileErrors::Error(Error::Other(
                "Coinbase transactions not supported".to_string(),
            )));
        }

        if response["vout"].as_array().unwrap().len() < vout {
            return Err(PrecompileErrors::Error(Error::Other(
                "Vout index out of bounds".to_string(),
            )));
        }

        if to_sats(response["vout"][vout]["value"].as_f64().unwrap()) < sat {
            return Err(PrecompileErrors::Error(Error::Other(
                "Sat value out of bounds".to_string(),
            )));
        }

        let new_pkscript = response["vout"][vout]["scriptPubKey"]["hex"]
            .as_str()
            .unwrap();

        let mut total_vout_sat_count = 0;
        let mut current_vout_index = 0;
        while current_vout_index < vout {
            let value = to_sats(
                response["vout"][current_vout_index]["value"]
                    .as_f64()
                    .unwrap(),
            );
            total_vout_sat_count += value;
            current_vout_index += 1;
        }
        total_vout_sat_count += sat;

        let mut total_vin_sat_count = 0;
        let mut current_vin_index = 0;
        let mut current_vin_txid = "".to_string();
        let mut current_vin_vout = 0;
        let mut current_vin_script_pub_key_hex = "".to_string();
        let mut current_vin_value = 0;
        let vin_count = response["vin"].as_array().unwrap().len();
        while total_vin_sat_count < total_vout_sat_count && current_vin_index < vin_count {
            current_vin_txid = response["vin"][current_vin_index]["txid"]
                .as_str()
                .unwrap()
                .to_string();
            current_vin_vout = response["vin"][current_vin_index]["vout"]
                .as_u64()
                .unwrap() as usize;
            gas_used += GAS_PER_RPC_CALL;
            if gas_used > gas_limit {
                return Err(PrecompileErrors::Error(Error::OutOfGas));
            }
            let vin_response = get_raw_transaction(&current_vin_txid);
            let vin_response = vin_response["result"].clone();
            current_vin_script_pub_key_hex = vin_response["vout"][current_vin_vout]
                ["scriptPubKey"]["hex"]
                .as_str()
                .unwrap()
                .to_string();
            current_vin_value = to_sats(
                vin_response["vout"][current_vin_vout]["value"]
                    .as_f64()
                    .unwrap(),
            );

            total_vin_sat_count += current_vin_value;
            current_vin_index += 1;
        }

        if total_vin_sat_count < total_vout_sat_count {
            return Err(PrecompileErrors::Error(Error::Other(
                "Insufficient satoshis in vin".to_string(),
            )));
        }

        let bytes = LAST_SAT_LOCATION.encode_returns(&(
            current_vin_txid,
            U256::from(current_vin_vout as u64),
            U256::from(total_vout_sat_count - (total_vin_sat_count - current_vin_value)),
            current_vin_script_pub_key_hex,
            new_pkscript.to_string(),
        ));

        Ok(PrecompileOutput {
            bytes: Bytes::from(bytes),
            gas_used,
        })
    }
}

fn to_sats(btc_value: f64) -> u64 {
    (btc_value * 1e8) as u64
}

#[cfg(test)]
mod tests {
    use revm::{ContextStatefulPrecompile, InnerEvmContext};
    use solabi::U256;

    use crate::evm::precompiles::LastSatLocationPrecompile;

    use super::LAST_SAT_LOCATION;

    #[test]
    fn test_get_last_sat_location_encode_params_single_vin_vout() {
        // https://mempool.space/testnet4/tx/cedfb4b62224a4782a4453dff73f3d48bb0d7da4d0f2238b0e949f9342de038a
        let txid = "cedfb4b62224a4782a4453dff73f3d48bb0d7da4d0f2238b0e949f9342de038a";
        let vout = U256::from(0u64);
        let sat = U256::from(100u64);
        let data = LAST_SAT_LOCATION.encode_params(&(txid.to_string(), vout, sat));

        // Consider mocking the RPC call to bitcoind
        let precompile = LastSatLocationPrecompile;
        let result = precompile.call(
            &data.into(),
            1000000,
            &mut InnerEvmContext::new(db::DB::default()),
        );
        let result = result.unwrap();
        let returns = LAST_SAT_LOCATION.decode_returns(&result.bytes).unwrap();

        assert_eq!(
            result.gas_used, 200000
        );

        assert_eq!(
            returns,
            (
                "3fc5d49bad92a1bc10c9b0981f13c47600af871ddc9bac15d22d2c05b6ae80f1".to_string(),
                U256::from(0u64),
                U256::from(100u64),
                "5120154a7cbf83a5ad929e224725b915ca5cd7ca7719a9ba2af90945a74e3431934d".to_string(),
                "5120fcdc5a7bd66b4d3a8c91f1a1cf94ad7d561f3a304bf18faf5678b1ee47e783b7".to_string()
            )
        );
    }

    #[test]
    fn test_get_last_sat_location_encode_params_multiple_vin_vout() {
        // https://mempool.space/testnet4/tx/581f13463e6a97b07b7643dc9bf741938b43bc468a10e918e48c5b8130051d09
        let txid = "581f13463e6a97b07b7643dc9bf741938b43bc468a10e918e48c5b8130051d09";
        let vout = U256::from(0u64);
        let sat = U256::from(100000u64);
        let data = LAST_SAT_LOCATION.encode_params(&(txid.to_string(), vout, sat));

        // Consider mocking the RPC call to bitcoind
        let precompile = LastSatLocationPrecompile;
        let result = precompile.call(
            &data.into(),
            10000000,
            &mut InnerEvmContext::new(db::DB::default()),
        );
        let result = result.unwrap();
        let returns = LAST_SAT_LOCATION.decode_returns(&result.bytes).unwrap();

        assert_eq!(
            result.gas_used, 400000
        );

        assert_eq!(
            returns,
            (
                "c5524694ce1664fe2e5221241177aa59c08cb96dcb063c81ef74eb6e23a1ae49".to_string(),
                U256::from(3u64),
                U256::from(18298u64),
                "5120b677a96a8bcf16d9c2b9e4af022654a4398306834f04eff883886a498cd8b47c".to_string(),
                "5120b40c065bfcc5962e1702f09de1a5d2dfc0a7236bbaf5c1672529b414b3ee4cf5".to_string()
            )
        );
    }

    #[test]
    fn test_coinbase_tx() {
        // https://mempool.space/testnet4/tx/ee7da837ec5807d2adc116c421488120da39f3eb72c8a07ec0e09583498b3ea8
        let txid = "ee7da837ec5807d2adc116c421488120da39f3eb72c8a07ec0e09583498b3ea8";
        let vout = U256::from(0u64);
        let sat = U256::from(100u64);
        let data = LAST_SAT_LOCATION.encode_params(&(txid.to_string(), vout, sat));

        // Consider mocking the RPC call to bitcoind
        let precompile = LastSatLocationPrecompile;
        let result = precompile.call(
            &data.into(),
            1000000,
            &mut InnerEvmContext::new(db::DB::default()),
        );
        let result = result.unwrap_err();

        assert_eq!(
            result.to_string(),
            "Coinbase transactions not supported"
        );
    }
}
