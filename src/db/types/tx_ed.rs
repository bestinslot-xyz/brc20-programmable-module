use std::error::Error;

use serde::{Deserialize, Serialize, Serializer};

use crate::db::types::{AddressED, BytesED, Decode, Encode, B256ED, U64ED, U8ED};
use crate::global::CHAIN_ID;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
/// Represents a transaction entry from the EVM.
///
/// Refer to [Ethereum JSON-RPC documentation on eth_getTransactionByHash](https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_gettransactionbyhash) for details
pub struct TxED {
    /// The hash of the transaction
    pub hash: B256ED,
    /// The nonce of the transaction
    pub nonce: U64ED,
    #[serde(rename = "blockHash")]
    /// The hash of the block that contains the transaction
    pub block_hash: B256ED,
    #[serde(rename = "blockNumber")]
    /// The number of the block that contains the transaction
    pub block_number: U64ED,
    #[serde(rename = "transactionIndex")]
    /// The index of the transaction in the block
    pub transaction_index: U64ED,
    /// The address of the sender
    pub from: AddressED,
    /// The address of the recipient, empty if the transaction is a contract creation
    pub to: Option<AddressED>,
    /// The value transferred in the transaction
    pub value: U64ED,
    /// The gas limit for the transaction
    pub gas: U64ED,
    #[serde(rename = "gasPrice")]
    /// The gas price for the transaction, 0 for BRC2.0
    pub gas_price: U64ED,
    /// The input data for the transaction
    pub input: BytesED,
    /// The v field of the transaction, 0 for BRC2.0
    pub v: U8ED,
    /// The r field of the transaction, 0 for BRC2.0
    pub r: U8ED,
    /// The s field of the transaction, 0 for BRC2.0
    pub s: U8ED,
    #[serde(rename = "chainId")]
    /// The chain ID for the transaction
    pub chain_id: U64ED,
    #[serde(
        rename = "type",
        serialize_with = "no_hex",
        deserialize_with = "no_hex_deserialize"
    )]
    /// The type of the transaction, always 0 for BRC2.0
    pub tx_type: U8ED,
    #[serde(skip_serializing, skip_deserializing)]
    /// The inscription ID that generated this transaction, if applicable
    pub inscription_id: Option<String>,
}

fn no_hex<S>(value: &U8ED, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u64(value.uint.as_limbs()[0])
}

fn no_hex_deserialize<'de, D>(deserializer: D) -> Result<U8ED, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = u8::deserialize(deserializer)?;
    Ok(U8ED::from(value))
}

#[cfg(feature = "server")]
impl TxED {
    // This is returned by the API, so doesn't need to be public
    pub(crate) fn new(
        hash: B256ED,
        nonce: U64ED,
        block_hash: B256ED,
        block_number: U64ED,
        transaction_index: U64ED,
        from: AddressED,
        to: Option<AddressED>,
        gas: U64ED,
        input: BytesED,
        inscription_id: Option<String>,
    ) -> Self {
        TxED {
            hash,
            nonce,
            block_hash,
            block_number,
            transaction_index,
            from,
            to,
            value: 0u64.into(),
            gas,
            gas_price: 0u64.into(),
            input,
            v: 0u8.into(),
            r: 0u8.into(),
            s: 0u8.into(),
            chain_id: CHAIN_ID.into(),
            tx_type: 0u8.into(),
            inscription_id,
        }
    }
}

impl Encode for TxED {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.hash.encode(buffer);
        self.nonce.encode(buffer);
        self.block_hash.encode(buffer);
        self.block_number.encode(buffer);
        self.transaction_index.encode(buffer);
        self.from.encode(buffer);
        self.to.encode(buffer);
        self.value.encode(buffer);
        self.gas.encode(buffer);
        self.gas_price.encode(buffer);
        self.input.encode(buffer);
        self.inscription_id.encode(buffer);
    }
}

impl Decode for TxED {
    fn decode(bytes: &[u8], offset: usize) -> Result<(Self, usize), Box<dyn Error>> {
        let (hash, offset) = Decode::decode(bytes, offset)?;
        let (nonce, offset) = Decode::decode(bytes, offset)?;
        let (block_hash, offset) = Decode::decode(bytes, offset)?;
        let (block_number, offset) = Decode::decode(bytes, offset)?;
        let (transaction_index, offset) = Decode::decode(bytes, offset)?;
        let (from, offset) = Decode::decode(bytes, offset)?;
        let (to, offset) = Decode::decode(bytes, offset)?;
        let (value, offset) = Decode::decode(bytes, offset)?;
        let (gas, offset) = Decode::decode(bytes, offset)?;
        let (gas_price, offset) = Decode::decode(bytes, offset)?;
        let (input, offset) = Decode::decode(bytes, offset)?;
        let (inscription_id, offset) = Decode::decode(bytes, offset)?;

        Ok((
            TxED {
                hash,
                nonce,
                block_hash,
                block_number,
                transaction_index,
                from,
                to,
                value,
                gas,
                gas_price,
                input,
                v: 0u8.into(),
                r: 0u8.into(),
                s: 0u8.into(),
                chain_id: CHAIN_ID.into(),
                tx_type: 0u8.into(),
                inscription_id,
            },
            offset,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode() {
        let tx = TxED {
            hash: [1u8; 32].into(),
            nonce: 1u64.into(),
            block_hash: [2u8; 32].into(),
            block_number: 2u64.into(),
            transaction_index: 3u64.into(),
            from: [3u8; 20].into(),
            to: Some([4u8; 20].into()),
            value: 4u64.into(),
            gas: 5u64.into(),
            gas_price: 6u64.into(),
            input: vec![7, 8, 9].into(),
            v: 0u8.into(),
            r: 0u8.into(),
            s: 0u8.into(),
            chain_id: CHAIN_ID.into(),
            tx_type: 0u8.into(),
            inscription_id: Some("inscription_id".to_string()),
        };
        let encoded = tx.encode_vec();
        let decoded = TxED::decode_vec(&encoded).unwrap();
        assert_eq!(tx, decoded);
    }

    #[test]
    fn serialize() {
        let tx = TxED {
            hash: [1u8; 32].into(),
            nonce: 1u64.into(),
            block_hash: [2u8; 32].into(),
            block_number: 2u64.into(),
            transaction_index: 3u64.into(),
            from: [3u8; 20].into(),
            to: Some([4u8; 20].into()),
            value: 4u64.into(),
            gas: 5u64.into(),
            gas_price: 6u64.into(),
            input: vec![7, 8, 9].into(),
            v: 0u8.into(),
            r: 0u8.into(),
            s: 0u8.into(),
            chain_id: CHAIN_ID.into(),
            tx_type: 0u8.into(),
            inscription_id: None,
        };
        let serialized = serde_json::to_string(&tx).unwrap();
        assert_eq!(
            serialized,
            "{\"hash\":\"0x0101010101010101010101010101010101010101010101010101010101010101\",\"nonce\":\"0x1\",\"blockHash\":\"0x0202020202020202020202020202020202020202020202020202020202020202\",\"blockNumber\":\"0x2\",\"transactionIndex\":\"0x3\",\"from\":\"0x0303030303030303030303030303030303030303\",\"to\":\"0x0404040404040404040404040404040404040404\",\"value\":\"0x4\",\"gas\":\"0x5\",\"gasPrice\":\"0x6\",\"input\":\"0x070809\",\"v\":\"0x0\",\"r\":\"0x0\",\"s\":\"0x0\",\"chainId\":\"0x4252433230\",\"type\":0}"
        );

        let deserialized: TxED = serde_json::from_str(&serialized).unwrap();
        assert_eq!(tx, deserialized);
    }

    #[test]
    fn deserialize_no_inscription_id() {
        let tx = TxED {
            hash: [1u8; 32].into(),
            nonce: 1u64.into(),
            block_hash: [2u8; 32].into(),
            block_number: 2u64.into(),
            transaction_index: 3u64.into(),
            from: [3u8; 20].into(),
            to: Some([4u8; 20].into()),
            value: 4u64.into(),
            gas: 5u64.into(),
            gas_price: 6u64.into(),
            input: vec![7, 8, 9].into(),
            v: 0u8.into(),
            r: 0u8.into(),
            s: 0u8.into(),
            chain_id: CHAIN_ID.into(),
            tx_type: 0u8.into(),
            inscription_id: Some("inscription_id".to_string()),
        };
        let serialized = serde_json::to_string(&tx).unwrap();
        assert_eq!(
            serialized,
            "{\"hash\":\"0x0101010101010101010101010101010101010101010101010101010101010101\",\"nonce\":\"0x1\",\"blockHash\":\"0x0202020202020202020202020202020202020202020202020202020202020202\",\"blockNumber\":\"0x2\",\"transactionIndex\":\"0x3\",\"from\":\"0x0303030303030303030303030303030303030303\",\"to\":\"0x0404040404040404040404040404040404040404\",\"value\":\"0x4\",\"gas\":\"0x5\",\"gasPrice\":\"0x6\",\"input\":\"0x070809\",\"v\":\"0x0\",\"r\":\"0x0\",\"s\":\"0x0\",\"chainId\":\"0x4252433230\",\"type\":0}"
        );

        let deserialized: TxED = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.inscription_id, None);
    }
}
