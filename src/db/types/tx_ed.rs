use alloy_primitives::Address;
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
    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.hash.encode());
        bytes.extend_from_slice(&self.nonce.encode());
        bytes.extend_from_slice(&self.block_hash.encode());
        bytes.extend_from_slice(&self.block_number.encode());
        bytes.extend_from_slice(&self.transaction_index.encode());
        bytes.extend_from_slice(&self.from.encode());
        bytes.extend_from_slice(&self.to.as_ref().unwrap_or(&Address::ZERO.into()).encode());
        bytes.extend_from_slice(&self.value.encode());
        bytes.extend_from_slice(&self.gas.encode());
        bytes.extend_from_slice(&self.gas_price.encode());

        let input_bytes = &self.input.encode();
        bytes.extend_from_slice(&(input_bytes.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&input_bytes);

        let inscription_bytes = self
            .inscription_id
            .as_ref()
            .map(|id| id.as_bytes())
            .unwrap_or(&[]);

        bytes.extend_from_slice(&(inscription_bytes.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&inscription_bytes);

        bytes
    }
}

impl Decode for TxED {
    fn decode(bytes: Vec<u8>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut i = 0;
        let hash = B256ED::decode(bytes[i..i + 32].to_vec())?;
        i += 32;
        let nonce = U64ED::decode(bytes[i..i + 8].try_into()?)?;
        i += 8;
        let block_hash = B256ED::decode(bytes[i..i + 32].to_vec())?;
        i += 32;
        let block_number = U64ED::decode(bytes[i..i + 8].try_into()?)?;
        i += 8;
        let transaction_index = U64ED::decode(bytes[i..i + 8].try_into()?)?;
        i += 8;
        let from = AddressED::decode(bytes[i..i + 20].to_vec())?;
        i += 20;
        let to = AddressED::decode(bytes[i..i + 20].to_vec())?;
        i += 20;
        let to = if to.is_zero() { None } else { Some(to) };
        let value = U64ED::decode(bytes[i..i + 8].try_into()?)?;
        i += 8;
        let gas = U64ED::decode(bytes[i..i + 8].try_into()?)?;
        i += 8;
        let gas_price = U64ED::decode(bytes[i..i + 8].try_into()?)?;
        i += 8;
        let input_len = u32::from_be_bytes(bytes[i..i + 4].try_into()?);
        i += 4;
        let input = BytesED::decode(bytes[i..i + input_len as usize].to_vec())?;
        i += input_len as usize;
        let inscription_len = u32::from_be_bytes(bytes[i..i + 4].try_into()?);
        i += 4;
        let inscription_id = if inscription_len > 0 {
            Some(String::from_utf8(
                bytes[i..i + inscription_len as usize].to_vec(),
            )?)
        } else {
            None
        };
        Ok(TxED {
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
        })
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
        let encoded = tx.encode();
        let decoded = TxED::decode(encoded).unwrap();
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
