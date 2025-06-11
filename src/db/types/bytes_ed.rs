use std::error::Error;

use alloy::primitives::Bytes;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::db::types::{Decode, Encode};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
/// Represents a variable-size byte array
pub struct BytesED {
    /// The byte array value
    pub bytes: Bytes,
}

impl BytesED {
    // This is returned by the API, so doesn't need to be public
    pub(crate) fn new(bytes: Bytes) -> Self {
        Self { bytes }
    }
}

impl From<Bytes> for BytesED {
    fn from(bytes: Bytes) -> Self {
        BytesED::new(bytes)
    }
}

impl From<Vec<u8>> for BytesED {
    fn from(value: Vec<u8>) -> Self {
        BytesED::new(Bytes::from(value))
    }
}

impl<'de> Deserialize<'de> for BytesED {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_string = String::deserialize(deserializer)?;
        Ok(BytesED {
            bytes: hex_string.parse().map_err(serde::de::Error::custom)?,
        })
    }
}

impl Serialize for BytesED {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{:x}", self.bytes))
    }
}

impl Encode for BytesED {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.bytes.to_vec().encode(buffer);
    }
}

impl Decode for BytesED {
    fn decode(bytes: &[u8], offset: usize) -> Result<(Self, usize), Box<dyn Error>>
    where
        Self: Sized,
    {
        Vec::<u8>::decode(bytes, offset).map(|(bytes, offset)| {
            (
                BytesED {
                    bytes: bytes.into(),
                },
                offset,
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_ed() {
        let bytes_ed: BytesED = vec![1, 2, 3, 4, 5].into();
        let encoded = bytes_ed.encode_vec();
        let decoded = BytesED::decode_vec(&encoded).unwrap();
        assert_eq!(bytes_ed, decoded);
    }

    #[test]
    fn test_bytes_ed_empty() {
        let bytes_ed: BytesED = vec![].into();
        let encoded = bytes_ed.encode_vec();
        let decoded = BytesED::decode_vec(&encoded).unwrap();
        assert_eq!(bytes_ed, decoded);
    }

    #[test]
    fn test_bytes_ed_large() {
        let bytes_ed: BytesED = vec![1; 1000].into();
        let encoded = bytes_ed.encode_vec();
        let decoded = BytesED::decode_vec(&encoded).unwrap();
        assert_eq!(bytes_ed, decoded);
    }
}
