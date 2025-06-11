#[cfg(feature = "server")]
use std::error::Error;

#[cfg(feature = "server")]
use alloy::primitives::hex::FromHex;
use alloy::primitives::{Bytes, B256};
#[cfg(feature = "server")]
use base64::prelude::BASE64_STANDARD_NO_PAD;
#[cfg(feature = "server")]
use base64::Engine;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_either::SingleOrVec;

#[cfg(feature = "server")]
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
    #[serde(alias = "input", alias = "data")]
    pub data: Option<EthBytes>,
}

impl EthCall {
    /// Creates a new EthCall with the given parameters
    pub fn new(from: Option<AddressED>, to: Option<AddressED>, data: EthBytes) -> Self {
        Self {
            from,
            to,
            data: Some(data),
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

#[derive(Debug, Clone, PartialEq, Eq)]
/// A wrapper for encoded bytes that can be serialized and deserialized.
/// This struct is used to handle the encoding and decoding of bytes in the BRC20 protocol.
pub struct InscriptionBytes(Option<String>);

impl InscriptionBytes {
    /// Creates a new InscriptionBytes instance with the given inner string.
    pub fn new(inner: String) -> Self {
        Self(Some(inner))
    }

    /// Returns the inner string if it exists, otherwise returns empty string.
    pub fn to_string(&self) -> String {
        self.0.clone().unwrap_or_default()
    }

    /// Creates a new InscriptionBytes from the given bytes and the block height.
    ///
    /// If the block height is above compression activation height, the bytes are encoded using either nada or zstd compression, depending on the block height.
    /// The resulting string is base64 encoded and can be used for the data field in brc20 indexer methods.
    #[cfg(feature = "server")]
    pub fn from_bytes(bytes: Bytes, block_height: u64) -> Result<Self, Box<dyn Error>> {
        if block_height < *COMPRESSION_ACTIVATION_HEIGHT.read() {
            return Ok(Self::new(format!("{}", bytes)));
        }
        let mut data = vec![];
        let nada_encoded = nada::encode(bytes.clone());
        let mut zstd_compressed = vec![0; CALLDATA_LIMIT];
        let zstd_length =
            zstd_safe::compress(zstd_compressed.as_mut_slice(), bytes.iter().as_slice(), 22)
                .map_err(|e| format!("Failed to compress with zstd: {}", e))?;
        // Pick compression method with the shortest length
        // 0x00 = uncompressed
        // 0x01 = nada
        // 0x02 = zstd
        if bytes.len() < nada_encoded.len() && bytes.len() < zstd_length {
            data.insert(0, 0x00);
            data.extend_from_slice(&bytes);
        } else if nada_encoded.len() < zstd_length {
            data.insert(0, 0x01);
            data.extend_from_slice(&nada_encoded);
        } else {
            data.insert(0, 0x02);
            data.extend_from_slice(&zstd_compressed[..zstd_length]);
        }
        let base64_encoded = BASE64_STANDARD_NO_PAD.encode(data);
        Ok(Self(Some(base64_encoded)))
    }

    /// Creates a new InscriptionBytes instance with no inner string (None).
    pub fn empty() -> Self {
        Self(None)
    }

    // This is used by the server, so doesn't need to be public
    #[cfg(feature = "server")]
    pub(crate) fn value(&self, block_height: u64) -> Option<Bytes> {
        self.0
            .as_ref()
            .and_then(|s| decode_bytes_from_inscription_data(s, block_height))
    }
}

impl Serialize for InscriptionBytes {
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

impl<'de> Deserialize<'de> for InscriptionBytes {
    fn deserialize<D>(deserializer: D) -> Result<InscriptionBytes, D::Error>
    where
        D: Deserializer<'de>,
    {
        let Ok(s) = String::deserialize(deserializer) else {
            return Ok(InscriptionBytes::empty());
        };
        Ok(InscriptionBytes::new(s))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
//// A wrapper for encoded bytes that can be serialized and deserialized.
/// This struct is used to handle the encoding and decoding of bytes in the BRC20 protocol.
///
/// It can be used to represent both the data and input fields in the EthCall struct.
/// It can also handle the case where the data is not present (None).
pub struct EthBytes(Option<String>);

impl EthBytes {
    /// Creates a new EthBytes instance with the given inner string.
    pub fn new(inner: String) -> Self {
        Self(Some(inner))
    }

    /// Returns the inner string if it exists, otherwise returns empty string.
    pub fn to_string(&self) -> String {
        self.0.clone().unwrap_or_default()
    }

    /// Creates a new EthBytes instance from the given bytes.
    /// The bytes are converted to a hex string with a 0x prefix.
    /// This is used for encoding the data and input fields in the EthCall struct.
    pub fn from_bytes(bytes: Bytes) -> Self {
        Self(Some(format!("0x{}", hex::encode(bytes))))
    }

    /// Creates a new EthBytes instance with no inner string (None).
    pub fn empty() -> Self {
        Self(None)
    }

    // This is used by the server, so doesn't need to be public
    #[cfg(feature = "server")]
    pub(crate) fn value(&self) -> Option<Bytes> {
        self.0.as_ref().and_then(|s| Bytes::from_hex(s).ok())
    }
}

impl Serialize for EthBytes {
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

impl<'de> Deserialize<'de> for EthBytes {
    fn deserialize<D>(deserializer: D) -> Result<EthBytes, D::Error>
    where
        D: Deserializer<'de>,
    {
        let Ok(s) = String::deserialize(deserializer) else {
            return Ok(EthBytes::empty());
        };
        Ok(EthBytes::new(s))
    }
}

#[cfg(feature = "server")]
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
                if base64_decoded.len() > CALLDATA_LIMIT {
                    None
                } else {
                    Some(Bytes::from(base64_decoded[1..].to_vec()))
                }
            }
            0x01 => {
                // Nada
                nada::decode_with_limit(base64_decoded[1..].iter().cloned(), CALLDATA_LIMIT)
                    .ok()
                    .map(Bytes::from)
            }
            0x02 => {
                // Zstd
                match zstd_safe::get_frame_content_size(&base64_decoded[1..]) {
                    Ok(Some(size)) => {
                        // Early exit if size is too large
                        if size > CALLDATA_LIMIT as u64 {
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

#[cfg(feature = "server")]
fn decode_zstd_into_bytes(data: &[u8]) -> Option<Bytes> {
    let mut decompressed = vec![0u8; CALLDATA_LIMIT]; // 1MB buffer
    if let Ok(length) = zstd_safe::decompress(decompressed.as_mut_slice(), &data) {
        if length > CALLDATA_LIMIT {
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
    use alloy::primitives::U256;
    use alloy::sol_types::{sol, SolCall};
    use base64::prelude::BASE64_STANDARD_NO_PAD;

    use super::*;
    use crate::global::COMPRESSION_ACTIVATION_HEIGHT;

    sol! {
        function transfer(address receiver, bytes ticker, uint256 amount);
    }

    #[test]
    fn test_calldata_inscription_roundtrip_before_compression_activation_height() {
        let address: &str = "0xdead09C7d1621C9D49EdD5c070933b500ac5beef";
        let ticker = vec![0x6f, 0x72, 0x64, 0x69];
        let amount = 42;
        let data = Bytes::from(
            transferCall::new((
                address.parse().unwrap(),
                Bytes::from(ticker),
                U256::from(amount),
            ))
            .abi_encode(),
        );

        let encoded = InscriptionBytes::from_bytes(data.clone(), 123456).unwrap();

        let decoded = encoded.value(123456).unwrap();

        assert_eq!(decoded, data);
    }

    #[test]
    fn test_calldata_inscription_roundtrip_at_compression_activation_height() {
        let address: &str = "0xdead09C7d1621C9D49EdD5c070933b500ac5beef";
        let ticker = vec![0x6f, 0x72, 0x64, 0x69];
        let amount = 42;
        let data = Bytes::from(
            transferCall::new((
                address.parse().unwrap(),
                Bytes::from(ticker),
                U256::from(amount),
            ))
            .abi_encode(),
        );

        let encoded =
            InscriptionBytes::from_bytes(data.clone(), *COMPRESSION_ACTIVATION_HEIGHT.read())
                .unwrap();

        let decoded = encoded
            .value(*COMPRESSION_ACTIVATION_HEIGHT.read())
            .unwrap();

        assert_eq!(decoded, data);
    }

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
