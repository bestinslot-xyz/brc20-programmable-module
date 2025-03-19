use bip322::verify_simple_encoded;
use revm::precompile::Error;
use revm::primitives::{Bytes, PrecompileErrors, PrecompileOutput, PrecompileResult};
use revm::{ContextStatefulPrecompile, InnerEvmContext};
use solabi::{selector, FunctionEncoder};

use crate::db::DB;

pub struct BIP322Precompile;

const VERIFY: FunctionEncoder<(String, String, String), (bool,)> =
    FunctionEncoder::new(selector!("verify(string,string,string)"));

impl ContextStatefulPrecompile<DB> for BIP322Precompile {
    fn call(
        &self,
        bytes: &Bytes,
        gas_limit: u64,
        _evmctx: &mut InnerEvmContext<DB>,
    ) -> PrecompileResult {
        let gas_used = 20000;
        let result = VERIFY.decode_params(&bytes);

        if result.is_err() {
            return Err(PrecompileErrors::Error(Error::Other(
                "Invalid params".to_string(),
            )));
        }

        let (address, message, signature) = result.unwrap();
        let result = verify_simple_encoded(&address, &message, &signature);

        if gas_used > gas_limit {
            return Err(PrecompileErrors::Error(Error::OutOfGas));
        }

        Ok(PrecompileOutput {
            bytes: Bytes::from(VERIFY.encode_returns(&(result.is_ok(),))),
            gas_used,
        })
    }
}

#[cfg(test)]
mod tests {
    use revm::primitives::Bytes;
    use revm::{ContextStatefulPrecompile, InnerEvmContext};

    use super::*;
    use crate::db::DB;

    #[test]
    fn test_verify() {
        let address = "bc1q9vza2e8x573nczrlzms0wvx3gsqjx7vavgkx0l";
        let message = "Hello World";
        let wif_private_key = "L3VFeEujGtevx9w18HD1fhRbCH67Az2dpCymeRE1SoPK6XQtaN2k";

        let signature = bip322::sign_simple_encoded(&address, &message, &wif_private_key).unwrap();

        let precompile = BIP322Precompile {};

        let bytes = VERIFY.encode_params(&(address.to_string(), message.to_string(), signature));

        let result = precompile
            .call(
                &Bytes::from_iter(bytes.iter()),
                1000000,
                &mut InnerEvmContext::new(DB::default()),
            )
            .unwrap();
        let (success,) = VERIFY.decode_returns(&result.bytes).unwrap();

        assert!(success);
    }
}
