use revm::primitives::{FixedBytes, B256};
use std::error::Error;

use super::{Decode, Encode};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BEncodeDecode<const N: usize>(pub FixedBytes<N>);
pub type B256ED = BEncodeDecode<32>;

impl B256ED {
    pub fn from_b256(a: B256) -> Self {
        Self(a)
    }
}

impl Encode for B256ED {
    fn encode(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.0.as_slice());
        Ok(bytes)
    }
}

impl Decode for B256ED {
    fn decode(bytes: Vec<u8>) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(BEncodeDecode(FixedBytes::from(arr)))
    }
}

#[cfg(test)]
mod tests {
    use revm::primitives::B256;

    use crate::types::{Decode, Encode, B256ED};

    #[test]
    fn test_b256_ed() {
        let b256: B256 = B256::from([0u8; 32]);
        let b256_ed = B256ED::from_b256(b256);
        let bytes = B256ED::encode(&b256_ed).unwrap();
        let decoded = B256ED::decode(bytes).unwrap();
        assert_eq!(b256_ed.0, decoded.0);
    }
}
