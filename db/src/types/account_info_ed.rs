use std::{borrow::Cow, error::Error};

use heed::{BytesDecode, BytesEncode};
use revm::primitives::{AccountInfo, B256, U256};

pub struct AccountInfoED(pub AccountInfo);

impl AccountInfoED {
    pub fn from_account_info(a: AccountInfo) -> Self {
        Self(a)
    }
}

impl<'a> BytesEncode<'a> for AccountInfoED {
    type EItem = AccountInfoED;

    fn bytes_encode(item: &'a Self::EItem) -> Result<Cow<'a, [u8]>, Box<dyn Error>> {
        let mut bytes = Vec::new();
        for limb in item.0.balance.as_limbs().iter() {
            bytes.extend_from_slice(&limb.to_be_bytes());
        }
        bytes.extend_from_slice(&item.0.nonce.to_be_bytes());
        bytes.extend_from_slice(&item.0.code_hash.0.to_vec());
        Ok(Cow::Owned(bytes))
    }
}

impl<'a> BytesDecode<'a> for AccountInfoED {
    type DItem = AccountInfoED;

    fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem, Box<dyn Error>> {
        let mut limbs = [0u64; 4];
        for (i, limb) in limbs.iter_mut().enumerate() {
            let start = i * 8;
            let end = start + 8;
            let bytes = &bytes[start..end];
            *limb = u64::from_be_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]);
        }
        let balance = U256::from_limbs(limbs);
        let nonce = u64::from_be_bytes([
            bytes[32], bytes[33], bytes[34], bytes[35], bytes[36], bytes[37], bytes[38], bytes[39],
        ]);
        let mut limbs = [0u64; 4];
        for (i, limb) in limbs.iter_mut().enumerate() {
            let start = (8 - i) * 8;
            let end = start + 8;
            let bytes = &bytes[start..end];
            *limb = u64::from_be_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]);
        }
        let code_hash_u = U256::from_limbs(limbs);
        let code_hash = B256::from(code_hash_u);
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
    use heed::{BytesDecode, BytesEncode};
    use revm::primitives::AccountInfo;
    use revm::primitives::B256;
    use revm::primitives::U256;

    use crate::types::AccountInfoED;

    #[test]
    fn test_account_info_ed() {
        // test by converting to bytes and decoding
        let account_info = AccountInfoED::from_account_info(AccountInfo {
            balance: U256::from(100),
            nonce: 1,
            code_hash: B256::from([1; 32]),
            code: None,
        });
        let bytes = AccountInfoED::bytes_encode(&account_info).unwrap();
        let decoded = AccountInfoED::bytes_decode(&bytes).unwrap();
        assert_eq!(account_info.0.balance, decoded.0.balance);
        assert_eq!(account_info.0.nonce, decoded.0.nonce);
        assert_eq!(account_info.0.code_hash, decoded.0.code_hash);
        assert_eq!(account_info.0.code, decoded.0.code);
    }
}
