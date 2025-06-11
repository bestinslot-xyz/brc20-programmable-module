use std::error::Error;

use alloy::primitives::FixedBytes;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::db::types::{Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Represents a fixed-size byte array of length N
///
/// Wrapper around `FixedBytes<N>` to provide serialization and deserialization
pub struct FixedBytesED<const N: usize> {
    /// The fixed-size byte array
    pub bytes: FixedBytes<N>,
}

/// Type alias for a 32-byte fixed-size byte array
pub type B256ED = FixedBytesED<32>;
/// Type alias for a 256-byte fixed-size byte array
pub type B2048ED = FixedBytesED<256>;

impl<const N: usize> FixedBytesED<N> {
    /// Creates a new `FixedBytesED` instance from a `FixedBytes<N>` instance.
    pub fn new(bytes: FixedBytes<N>) -> Self {
        Self { bytes }
    }
}

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
        S: Serializer,
    {
        let hex_string = format!("0x{:x}", self.bytes);
        serializer.serialize_str(&hex_string)
    }
}

impl<'de, const N: usize> Deserialize<'de> for FixedBytesED<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_string = String::deserialize(deserializer)?;
        Ok(FixedBytesED {
            bytes: hex_string.parse().map_err(serde::de::Error::custom)?,
        })
    }
}

impl<const N: usize> TryFrom<&str> for FixedBytesED<N> {
    type Error = Box<dyn Error>;

    fn try_from(hex_string: &str) -> Result<Self, Self::Error> {
        let bytes = hex_string.trim_start_matches("0x");
        let bytes = hex::decode(bytes)?;
        if bytes.len() != N {
            return Err("Invalid length".into());
        }
        Ok(FixedBytesED {
            bytes: FixedBytes::from_slice(&bytes),
        })
    }
}

impl<const N: usize> Encode for FixedBytesED<N> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(self.bytes.as_slice());
    }
}

impl<const N: usize> Decode for FixedBytesED<N> {
    fn decode(bytes: &[u8], offset: usize) -> Result<(Self, usize), Box<dyn Error>>
    where
        Self: Sized,
    {
        <[u8; N]>::decode(bytes, offset).map(|(bytes, offset)| (bytes.into(), offset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_b256_ed() {
        let b256_ed: B256ED = [0u8; 32].into();
        let bytes = b256_ed.encode_vec();
        let decoded = B256ED::decode_vec(&bytes).unwrap();
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

    #[test]
    fn test_b256_ed_deserialize() {
        let serialized = "\"0x0101010101010101010101010101010101010101010101010101010101010101\"";
        let deserialized: B256ED = serde_json::from_str(serialized).unwrap();
        assert_eq!(deserialized.bytes, [1u8; 32]);
    }
}
