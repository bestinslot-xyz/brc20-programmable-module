use std::error::Error;

use alloy_primitives::Bytes;
use serde::Serialize;

use crate::db::types::{Decode, Encode};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct BytesED {
    pub bytes: Bytes,
}

impl From<Bytes> for BytesED {
    fn from(bytes: Bytes) -> Self {
        Self { bytes }
    }
}

impl From<Vec<u8>> for BytesED {
    fn from(value: Vec<u8>) -> Self {
        Self {
            bytes: Bytes::from(value),
        }
    }
}

impl Serialize for BytesED {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{:x}", self.bytes))
    }
}

impl Encode for BytesED {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.bytes.iter().as_slice());
        bytes
    }
}

impl Decode for BytesED {
    fn decode(bytes: Vec<u8>) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        Ok(Self {
            bytes: Bytes::from(bytes),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_ed() {
        let bytes_ed: BytesED = vec![1, 2, 3, 4, 5].into();
        let encoded = bytes_ed.encode();
        let decoded = BytesED::decode(encoded).unwrap();
        assert_eq!(bytes_ed, decoded);
    }

    #[test]
    fn test_bytes_ed_empty() {
        let bytes_ed: BytesED = vec![].into();
        let encoded = bytes_ed.encode();
        let decoded = BytesED::decode(encoded).unwrap();
        assert_eq!(bytes_ed, decoded);
    }

    #[test]
    fn test_bytes_ed_large() {
        let bytes_ed: BytesED = vec![1; 1000].into();
        let encoded = bytes_ed.encode();
        let decoded = BytesED::decode(encoded).unwrap();
        assert_eq!(bytes_ed, decoded);
    }
}
