use std::error::Error;

use alloy_primitives::Bytes;
use serde::Serialize;

use crate::db::types::{Decode, Encode};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct BytesED(pub Bytes);

impl Serialize for BytesED {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{:x}", self.0))
    }
}

impl Encode for BytesED {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.0.iter().as_slice());
        bytes
    }
}

impl Decode for BytesED {
    fn decode(bytes: Vec<u8>) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        Ok(BytesED(bytes.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_ed() {
        let bytes = Bytes::from(vec![1, 2, 3, 4, 5]);
        let bytes_ed = BytesED(bytes.clone());
        let encoded = bytes_ed.encode();
        let decoded = BytesED::decode(encoded).unwrap();
        assert_eq!(bytes_ed.0, decoded.0);
    }

    #[test]
    fn test_bytes_ed_empty() {
        let bytes = Bytes::from(vec![]);
        let bytes_ed = BytesED(bytes.clone());
        let encoded = bytes_ed.encode();
        let decoded = BytesED::decode(encoded).unwrap();
        assert_eq!(bytes_ed.0, decoded.0);
    }

    #[test]
    fn test_bytes_ed_large() {
        let bytes = Bytes::from(vec![1; 1000]);
        let bytes_ed = BytesED(bytes.clone());
        let encoded = bytes_ed.encode();
        let decoded = BytesED::decode(encoded).unwrap();
        assert_eq!(bytes_ed.0, decoded.0);
    }
}
