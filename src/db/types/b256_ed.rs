use std::error::Error;

use alloy_primitives::FixedBytes;
use serde::Serialize;

use crate::db::types::{Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FixedBytesED<const N: usize> {
    pub bytes: FixedBytes<N>,
}

pub type B256ED = FixedBytesED<32>;
pub type B2048ED = FixedBytesED<256>;

impl<const N: usize> From<FixedBytes<N>> for FixedBytesED<N> {
    fn from(bytes: FixedBytes<N>) -> Self {
        Self { bytes }
    }
}

impl<const N: usize> From<[u8; N]> for FixedBytesED<N> {
    fn from(bytes: [u8; N]) -> Self {
        Self {
            bytes: bytes.into(),
        }
    }
}

impl<const N: usize> From<FixedBytesED<N>> for FixedBytes<N> {
    fn from(b_encode_decode: FixedBytesED<N>) -> Self {
        b_encode_decode.bytes
    }
}

impl<const N: usize> Serialize for FixedBytesED<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let hex_string = format!("0x{:x}", self.bytes);
        serializer.serialize_str(&hex_string)
    }
}

impl<const N: usize> Encode for FixedBytesED<N> {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.bytes.as_slice());
        bytes
    }
}

impl<const N: usize> Decode for FixedBytesED<N> {
    fn decode(bytes: Vec<u8>) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let mut slice = [0u8; N];
        slice.copy_from_slice(&bytes);
        Ok(Self {
            bytes: slice.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_b256_ed() {
        let b256_ed: B256ED = [0u8; 32].into();
        let bytes = b256_ed.encode();
        let decoded = B256ED::decode(bytes).unwrap();
        assert_eq!(b256_ed, decoded);
    }

    #[test]
    fn test_b256_ed_serialize() {
        let b256_ed: B256ED = [1u8; 32].into();
        let serialized = serde_json::to_string(&b256_ed).unwrap();
        assert_eq!(
            serialized,
            "\"0x0101010101010101010101010101010101010101010101010101010101010101\""
        );
    }
}
