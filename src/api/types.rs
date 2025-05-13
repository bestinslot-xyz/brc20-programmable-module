use alloy_primitives::hex::FromHex;
use alloy_primitives::{Bytes, B256};
use base64::prelude::BASE64_STANDARD_NO_PAD;
use base64::Engine;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_either::SingleOrVec;

use crate::global::{CALLDATA_LIMIT, COMPRESSION_ACTIVATION_HEIGHT};
use crate::types::{AddressED, B256ED};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents a call to a contract with optional parameters for from, to, data, and input.
pub struct EthCall {
    /// The address of the sender
    pub from: Option<AddressED>,
    /// The address of the contract to call
    pub to: Option<AddressED>,
    /// The data to send with the call
    pub data: Option<EncodedBytes>,
    /// The input data for the call (alternative to data, if both are present, data is used)
    pub input: Option<EncodedBytes>,
}

impl EthCall {
    /// Creates a new EthCall with the given parameters
    pub fn new(from: Option<AddressED>, to: Option<AddressED>, data: EncodedBytes) -> Self {
        Self {
            from,
            to,
            data: Some(data),
            input: None,
        }
    }

    // This is used by the server, so doesn't need to be public
    pub(crate) fn data_or_input(&self) -> Option<&EncodedBytes> {
        if let Some(data) = &self.data {
            Some(data)
        } else if let Some(input) = &self.input {
            Some(input)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents a filter for retrieving logs from the blockchain.
pub struct GetLogsFilter {
    #[serde(rename = "fromBlock")]
    /// The block number to start searching from
    pub from_block: Option<String>,
    #[serde(rename = "toBlock")]
    /// The block number to stop searching at
    pub to_block: Option<String>,
    /// The address of the contract to filter logs from
    pub address: Option<AddressED>,
    /// The topics to filter logs by
    pub topics: Option<Vec<SingleOrVec<Option<B256ED>>>>,
}

impl GetLogsFilter {
    // This is used by the server, so doesn't need to be public
    pub(crate) fn topics_as_b256(&self) -> Option<Vec<SingleOrVec<Option<B256>>>> {
        self.topics.as_ref().map(|topics| {
            topics
                .iter()
                .map(|topic| match topic {
                    SingleOrVec::Single(t) => SingleOrVec::Single(t.clone().map(|t| t.bytes)),
                    SingleOrVec::Vec(ts) => SingleOrVec::Vec(
                        ts.iter()
                            .map(|t| t.clone().map(|t| t.bytes))
                            .collect::<Vec<Option<B256>>>(),
                    ),
                })
                .collect()
        })
    }
}

#[derive(Debug, Clone)]
//// A wrapper for encoded bytes that can be serialized and deserialized.
/// This struct is used to handle the encoding and decoding of bytes in the BRC20 protocol.
///
/// It can be used to represent both the data and input fields in the EthCall struct.
/// It can also handle the case where the data is not present (None).
pub struct EncodedBytes(Option<String>);

impl EncodedBytes {
    /// Creates a new EncodedBytesWrapper with the given inner string.
    pub fn new(inner: String) -> Self {
        Self(Some(inner))
    }

    /// Creates a new EncodedBytesWrapper from the given bytes.
    /// The bytes are converted to a hex string with a 0x prefix.
    /// This is used for encoding the data and input fields in the EthCall struct.
    pub fn from_bytes(bytes: Bytes) -> Self {
        Self(Some(format!("0x{}", hex::encode(bytes))))
    }

    /// Creates a new EncodedBytesWrapper with no inner string (None).
    pub fn empty() -> Self {
        Self(None)
    }

    // This is used by the server, so doesn't need to be public
    pub(crate) fn value_inscription(&self, block_height: u64) -> Option<Bytes> {
        self.0
            .as_ref()
            .and_then(|s| decode_bytes_from_inscription_data(s, block_height))
    }

    // This is used by the server, so doesn't need to be public
    pub(crate) fn value_eth(&self) -> Option<Bytes> {
        self.0.as_ref().and_then(|s| Bytes::from_hex(s).ok())
    }
}

impl Serialize for EncodedBytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(inner) = &self.0 {
            serializer.serialize_str(inner)
        } else {
            serializer.serialize_none()
        }
    }
}

impl<'de> Deserialize<'de> for EncodedBytes {
    fn deserialize<D>(deserializer: D) -> Result<EncodedBytes, D::Error>
    where
        D: Deserializer<'de>,
    {
        let Ok(s) = String::deserialize(deserializer) else {
            return Ok(EncodedBytes::empty());
        };
        Ok(EncodedBytes::new(s))
    }
}

pub fn decode_bytes_from_inscription_data(
    inscription_data: &String,
    block_height: u64,
) -> Option<Bytes> {
    // Starting from compression_activation_height, we can use base64 encoding and compression
    if block_height < *COMPRESSION_ACTIVATION_HEIGHT.read() {
        Bytes::from_hex(inscription_data).ok()
    } else {
        let base64_decoded = BASE64_STANDARD_NO_PAD.decode(inscription_data).ok()?;
        // Use first byte to determine compression method
        // 0x00 = uncompressed
        // 0x01 = nada
        // 0x02 = zstd
        match base64_decoded[0] {
            0x00 => {
                // Uncompressed
                if base64_decoded.len() > *CALLDATA_LIMIT {
                    None
                } else {
                    Some(Bytes::from(base64_decoded[1..].to_vec()))
                }
            }
            0x01 => {
                // Nada
                nada::decode_with_limit(base64_decoded[1..].iter().cloned(), *CALLDATA_LIMIT)
                    .ok()
                    .map(Bytes::from)
            }
            0x02 => {
                // Zstd
                match zstd_safe::get_frame_content_size(&base64_decoded[1..]) {
                    Ok(Some(size)) => {
                        // Early exit if size is too large
                        if size > *CALLDATA_LIMIT as u64 {
                            return None;
                        }
                    }
                    Ok(None) => {
                        // No size information available, proceed with decompression anyway, it will fail eventually
                        // This is a valid case, as zstd can be used without size information
                    }
                    Err(_) => {
                        return None;
                    }
                }
                // In a separate method to avoid unnecessary stack allocation for zstd
                decode_zstd_into_bytes(&base64_decoded[1..])
            }
            _ => {
                // Unknown compression method
                None
            }
        }
    }
}

fn decode_zstd_into_bytes(data: &[u8]) -> Option<Bytes> {
    let mut decompressed: [u8; 1024 * 1024] = [0; 1024 * 1024]; // 1MB buffer
    if let Ok(length) = zstd_safe::decompress(&mut decompressed, &data) {
        if length > *CALLDATA_LIMIT {
            None
        } else {
            let mut decompressed = decompressed.to_vec();
            decompressed.truncate(length);
            Some(Bytes::from(decompressed))
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use base64::prelude::BASE64_STANDARD_NO_PAD;

    use super::*;
    use crate::global::COMPRESSION_ACTIVATION_HEIGHT;

    #[test]
    fn test_decode_bytes_from_inscription_data_old() {
        let inscription_data = "0xdeadbeef".to_string();
        let block_height = 123456;
        let result = decode_bytes_from_inscription_data(&inscription_data, block_height);
        assert_eq!(result, Some(Bytes::from(vec![0xde, 0xad, 0xbe, 0xef])));
    }

    #[test]
    fn test_decode_bytes_from_inscription_data_uncompressed() {
        // 0x00 to indicate uncompressed
        let data = vec![0x00, 0xde, 0xad, 0xbe, 0xef, 0xff];
        let base64_encoded = BASE64_STANDARD_NO_PAD.encode(data);
        let block_height = *COMPRESSION_ACTIVATION_HEIGHT.read();
        let result = decode_bytes_from_inscription_data(&base64_encoded, block_height);
        assert_eq!(
            result,
            Some(Bytes::from(vec![0xde, 0xad, 0xbe, 0xef, 0xff]))
        );
    }

    #[test]
    fn test_decode_bytes_from_inscription_data_nada() {
        let data = vec![
            0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff,
        ];
        let mut nada_encoded = nada::encode(data.clone());
        // Prepend 0x01 to indicate nada compression
        nada_encoded.insert(0, 0x01);
        assert_eq!(
            nada_encoded,
            vec![0x01, 0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0xff, 0x01, 0xff, 0x04, 0xff, 0x02]
        );

        let base64_encoded = BASE64_STANDARD_NO_PAD.encode(nada_encoded);
        let block_height = *COMPRESSION_ACTIVATION_HEIGHT.read();
        let result = decode_bytes_from_inscription_data(&base64_encoded, block_height);
        assert_eq!(result, Some(Bytes::from(data)));
    }

    #[test]
    fn test_decode_bytes_from_inscription_data_repetition_zstd() {
        // Repeated data is better for zstd compression, this is a test for that
        let data = vec![0xde, 0xad, 0xbe, 0xef].repeat(4096);
        let mut compressed = [0u8; 1024 * 1024];
        let length = zstd_safe::compress(&mut compressed, data.as_slice(), 22).unwrap();
        // Prepend 0x02 to indicate zstd compression
        let mut compressed_vec = compressed.to_vec();
        compressed_vec.truncate(length);
        compressed_vec.insert(0, 0x02);
        let base64_encoded = BASE64_STANDARD_NO_PAD.encode(compressed_vec);
        let block_height = *COMPRESSION_ACTIVATION_HEIGHT.read();
        let result = decode_bytes_from_inscription_data(&base64_encoded, block_height);
        assert_eq!(result, Some(Bytes::from(data)));
    }

    #[test]
    fn test_decode_bytes_from_inscription_data_random_zstd() {
        // Generate random data of size 32k, this performs very poorly with zstd, for testing purposes
        // Real life data will be much smaller and more compressible
        let mut data = vec![0; 32 * 1024];
        for i in 0..data.len() {
            data[i] = rand::random();
        }
        let mut compressed = [0u8; 1024 * 1024];
        let length = zstd_safe::compress(&mut compressed, data.as_slice(), 22).unwrap();
        // Prepend 0x02 to indicate zstd compression
        let mut compressed_vec = compressed.to_vec();
        compressed_vec.truncate(length);
        compressed_vec.insert(0, 0x02);
        let base64_encoded = BASE64_STANDARD_NO_PAD.encode(compressed_vec);
        let block_height = *COMPRESSION_ACTIVATION_HEIGHT.read();
        let result = decode_bytes_from_inscription_data(&base64_encoded, block_height);
        assert_eq!(result, Some(Bytes::from(data)));
    }

    #[test]
    fn test_decode_bytes_from_inscription_data_huge_zstd() {
        // Generate repeated data of size 2MB, this should be rejected by the decoder, but also not crash and burn
        let data = vec![0xde, 0xad, 0xbe, 0xef].repeat(512 * 1024);
        let mut compressed = [0u8; 1024 * 1024];
        let length = zstd_safe::compress(&mut compressed, data.as_slice(), 22).unwrap();
        // Prepend 0x02 to indicate zstd compression
        let mut compressed_vec = compressed.to_vec();
        compressed_vec.truncate(length);
        compressed_vec.insert(0, 0x02);
        // This is compressed to around 200 bytes
        let base64_encoded = BASE64_STANDARD_NO_PAD.encode(compressed_vec);
        let block_height = *COMPRESSION_ACTIVATION_HEIGHT.read();
        let result = decode_bytes_from_inscription_data(&base64_encoded, block_height);
        assert_eq!(result, None);
    }

    #[test]
    fn test_invalid_first_byte() {
        let data = vec![0x02, 0xde, 0xad, 0xbe, 0xef];
        let base64_encoded = BASE64_STANDARD_NO_PAD.encode(data);
        let block_height = *COMPRESSION_ACTIVATION_HEIGHT.read();
        let result = decode_bytes_from_inscription_data(&base64_encoded, block_height);
        assert_eq!(result, None);
    }

    #[test]
    fn test_invalid_base64() {
        let invalid_base64 = "invalid_base64";
        let block_height = *COMPRESSION_ACTIVATION_HEIGHT.read();
        let result = decode_bytes_from_inscription_data(&invalid_base64.to_string(), block_height);
        assert_eq!(result, None);
    }
}
