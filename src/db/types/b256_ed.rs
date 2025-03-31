use std::error::Error;

use revm::primitives::{FixedBytes, B256};
use serde::Serialize;

use crate::db::types::{Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BEncodeDecode<const N: usize>(pub FixedBytes<N>);
pub type B256ED = BEncodeDecode<32>;
pub type B2048ED = BEncodeDecode<256>;

impl B256ED {
    pub fn from_b256(a: B256) -> Self {
        Self(a)
    }
}

impl<const N: usize> Serialize for BEncodeDecode<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let hex_string = format!("0x{:x}", self.0);
        serializer.serialize_str(&hex_string)
    }
}

impl<const N: usize> Encode for BEncodeDecode<N> {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.0.as_slice());
        bytes
    }
}

impl<const N: usize> Decode for BEncodeDecode<N> {
    fn decode(bytes: Vec<u8>) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let mut bytes_array = [0u8; N];
        bytes_array.copy_from_slice(&bytes);
        Ok(BEncodeDecode(FixedBytes(bytes_array)))
    }
}

#[cfg(test)]
mod tests {
    use revm::primitives::B256;

    use super::*;

    #[test]
    fn test_b256_ed() {
        let b256: B256 = B256::from([0u8; 32]);
        let b256_ed = B256ED::from_b256(b256);
        let bytes = b256_ed.encode();
        let decoded = B256ED::decode(bytes).unwrap();
        assert_eq!(b256_ed.0, decoded.0);
    }

    #[test]
    fn test_b256_ed_serialize() {
        let b256: B256 = B256::from([1u8; 32]);
        let b256_ed = B256ED::from_b256(b256);
        let serialized = serde_json::to_string(&b256_ed).unwrap();
        assert_eq!(
            serialized,
            "\"0x0101010101010101010101010101010101010101010101010101010101010101\""
        );
    }
}
