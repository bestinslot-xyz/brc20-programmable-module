use std::error::Error;

use revm::primitives::ruint::aliases::U64;
use revm::primitives::U256;
use revm_state::AccountInfo;

use crate::db::types::{Decode, Encode};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct AccountInfoED(pub AccountInfo);

impl Encode for AccountInfoED {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.0.balance.to_be_bytes::<32>());
        bytes.extend_from_slice(&self.0.nonce.to_be_bytes());
        bytes.extend_from_slice(&self.0.code_hash.0.to_vec());
        bytes
    }
}

impl Decode for AccountInfoED {
    fn decode(bytes: Vec<u8>) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let balance = U256::from_be_bytes::<32>(bytes[0..32].try_into().unwrap());
        let nonce = U64::from_be_bytes::<8>(bytes[32..40].try_into().unwrap())
            .try_into()
            .unwrap();
        let code_hash_u = U256::from_be_bytes::<32>(bytes[40..72].try_into().unwrap());
        let code_hash = code_hash_u.into();
        Ok(AccountInfoED(AccountInfo {
            balance,
            nonce,
            code_hash,
            code: None,
        }))
    }
}

#[cfg(test)]
mod tests {
    use revm::primitives::U256;

    use super::*;

    #[test]
    fn test_account_info_ed() {
        let account_info = AccountInfoED(AccountInfo {
            balance: U256::from(100),
            nonce: 1,
            code_hash: [1; 32].into(),
            code: None,
        });
        let bytes = account_info.encode();
        let decoded = AccountInfoED::decode(bytes).unwrap();
        assert_eq!(account_info.0.balance, decoded.0.balance);
        assert_eq!(account_info.0.nonce, decoded.0.nonce);
        assert_eq!(account_info.0.code_hash, decoded.0.code_hash);
        assert_eq!(account_info.0.code, decoded.0.code);
    }
}
