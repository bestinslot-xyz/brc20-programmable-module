use std::error::Error;

use serde::Serialize;
use serde_hex::{CompactPfx, SerHex};

use super::U64ED;
use crate::db::types::{AddressED, BytesED, Decode, Encode, B256ED};
use crate::server::api::CHAIN_ID;

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct TxED {
    pub hash: B256ED,
    pub nonce: U64ED,
    #[serde(rename = "blockHash")]
    pub block_hash: B256ED,
    #[serde(rename = "blockNumber")]
    pub block_number: U64ED,
    #[serde(rename = "transactionIndex")]
    pub transaction_index: U64ED,
    pub from: AddressED,
    pub to: Option<AddressED>,
    pub value: U64ED,
    pub gas: U64ED,
    #[serde(rename = "gasPrice")]
    pub gas_price: U64ED,
    pub input: BytesED,
    #[serde(with = "SerHex::<CompactPfx>")]
    pub v: u8,
    #[serde(with = "SerHex::<CompactPfx>")]
    pub r: u8,
    #[serde(with = "SerHex::<CompactPfx>")]
    pub s: u8,
    #[serde(rename = "chainId")]
    pub chain_id: U64ED,
    #[serde(rename = "type")]
    pub tx_type: u8,
    #[serde(skip_serializing)]
    pub inscription_id: Option<String>,
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
                v: 0,
                r: 0,
                s: 0,
                chain_id: (*CHAIN_ID).into(),
                tx_type: 0,
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
            v: 0,
            r: 0,
            s: 0,
            chain_id: (*CHAIN_ID).into(),
            tx_type: 0,
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
            v: 0,
            r: 0,
            s: 0,
            chain_id: (*CHAIN_ID).into(),
            tx_type: 0,
            inscription_id: Some("inscription_id".to_string()),
        };
        let serialized = serde_json::to_string(&tx).unwrap();
        assert_eq!(
            serialized,
            "{\"hash\":\"0x0101010101010101010101010101010101010101010101010101010101010101\",\"nonce\":\"0x1\",\"blockHash\":\"0x0202020202020202020202020202020202020202020202020202020202020202\",\"blockNumber\":\"0x2\",\"transactionIndex\":\"0x3\",\"from\":\"0x0303030303030303030303030303030303030303\",\"to\":\"0x0404040404040404040404040404040404040404\",\"value\":\"0x4\",\"gas\":\"0x5\",\"gasPrice\":\"0x6\",\"input\":\"0x070809\",\"v\":\"0x0\",\"r\":\"0x0\",\"s\":\"0x0\",\"chainId\":\"0x4252433230\",\"type\":0}"
        )
    }
}
