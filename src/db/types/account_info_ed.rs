use std::error::Error;

use revm_state::AccountInfo;

use crate::db::types::{Decode, Encode, B256ED, U256ED, U64ED};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct AccountInfoED {
    pub balance: U256ED,
    pub nonce: U64ED,
    pub code_hash: B256ED,
}

impl From<AccountInfo> for AccountInfoED {
    fn from(account_info: AccountInfo) -> Self {
        AccountInfoED {
            balance: account_info.balance.into(),
            nonce: account_info.nonce.into(),
            code_hash: account_info.code_hash.into(),
        }
    }
}

impl Into<AccountInfo> for AccountInfoED {
    fn into(self) -> AccountInfo {
        AccountInfo {
            balance: self.balance.uint,
            nonce: self.nonce.into(),
            code_hash: self.code_hash.into(),
            code: None,
        }
    }
}

impl Encode for AccountInfoED {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.balance.encode());
        bytes.extend_from_slice(&self.nonce.encode());
        bytes.extend_from_slice(&self.code_hash.encode());
        bytes
    }
}

impl Decode for AccountInfoED {
    fn decode(bytes: Vec<u8>) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let balance = U256ED::decode(bytes[0..32].try_into()?).unwrap();
        let nonce = U64ED::decode(bytes[32..40].try_into()?).unwrap();
        let code_hash = B256ED::decode(bytes[40..72].try_into()?).unwrap();
        Ok(AccountInfoED {
            balance,
            nonce,
            code_hash,
        })
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::U256;

    use super::*;

    #[test]
    fn test_account_info_ed() {
        let account_info: AccountInfoED = AccountInfo {
            balance: U256::from(100),
            nonce: 1,
            code_hash: [1; 32].into(),
            code: None,
        }
        .into();
        let bytes = account_info.encode();
        let decoded = AccountInfoED::decode(bytes).unwrap();
        assert_eq!(account_info.balance, decoded.balance);
        assert_eq!(account_info.nonce, decoded.nonce);
        assert_eq!(account_info.code_hash, decoded.code_hash);
    }
}
