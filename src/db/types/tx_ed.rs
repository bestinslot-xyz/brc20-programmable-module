use revm::primitives::{hex, Address, Bytes};
use serde::Serialize;
use serde_hex::{CompactPfx, SerHex};

use crate::db::types::{AddressED, Decode, Encode, B256ED};
use crate::server::CHAIN_ID;

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct TxED {
    pub hash: B256ED,
    #[serde(with = "SerHex::<CompactPfx>")]
    pub nonce: u64,
    #[serde(rename = "blockHash")]
    pub block_hash: B256ED,
    #[serde(rename = "blockNumber", with = "SerHex::<CompactPfx>")]
    pub block_number: u64,
    #[serde(rename = "transactionIndex", with = "SerHex::<CompactPfx>")]
    pub transaction_index: u64,
    pub from: AddressED,
    pub to: Option<AddressED>,
    #[serde(with = "SerHex::<CompactPfx>")]
    pub value: u64,
    #[serde(with = "SerHex::<CompactPfx>")]
    pub gas: u64,
    #[serde(rename = "gasPrice", with = "SerHex::<CompactPfx>")]
    pub gas_price: u64,
    #[serde(serialize_with = "bytes_to_hex")]
    pub input: Bytes,
    #[serde(with = "SerHex::<CompactPfx>")]
    pub v: u8,
    #[serde(with = "SerHex::<CompactPfx>")]
    pub r: u8,
    #[serde(with = "SerHex::<CompactPfx>")]
    pub s: u8,
    #[serde(rename = "chainId")]
    pub chain_id: u64,
    #[serde(rename = "type")]
    pub tx_type: u8,
}

fn bytes_to_hex<S>(bytes: &Bytes, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&format!("0x{}", &hex::encode(&bytes)))
}

impl Encode for TxED {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.hash.encode());
        bytes.extend_from_slice(&self.nonce.to_be_bytes());
        bytes.extend_from_slice(&self.block_hash.encode());
        bytes.extend_from_slice(&self.block_number.to_be_bytes());
        bytes.extend_from_slice(&self.transaction_index.to_be_bytes());
        bytes.extend_from_slice(&self.from.encode());
        bytes.extend_from_slice(
            &self
                .to
                .as_ref()
                .unwrap_or(&AddressED(Address::ZERO))
                .encode(),
        );
        bytes.extend_from_slice(&self.value.to_be_bytes());
        bytes.extend_from_slice(&self.gas.to_be_bytes());
        bytes.extend_from_slice(&self.gas_price.to_be_bytes());
        bytes.extend_from_slice(&(self.input.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&self.input as &[u8]);
        bytes
    }
}

impl Decode for TxED {
    fn decode(bytes: Vec<u8>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut i = 0;
        let hash = B256ED::decode(bytes[i..i + 32].to_vec())?;
        i += 32;
        let nonce = u64::from_be_bytes(bytes[i..i + 8].try_into()?);
        i += 8;
        let block_hash = B256ED::decode(bytes[i..i + 32].to_vec())?;
        i += 32;
        let block_number = u64::from_be_bytes(bytes[i..i + 8].try_into()?);
        i += 8;
        let transaction_index = u64::from_be_bytes(bytes[i..i + 8].try_into()?);
        i += 8;
        let from = AddressED::decode(bytes[i..i + 20].to_vec())?;
        i += 20;
        let to = AddressED::decode(bytes[i..i + 20].to_vec())?;
        i += 20;
        let to = if to.0 == Address::ZERO {
            None
        } else {
            Some(to)
        };
        let value = u64::from_be_bytes(bytes[i..i + 8].try_into()?);
        i += 8;
        let gas = u64::from_be_bytes(bytes[i..i + 8].try_into()?);
        i += 8;
        let gas_price = u64::from_be_bytes(bytes[i..i + 8].try_into()?);
        i += 8;
        let input_len = u32::from_be_bytes(bytes[i..i + 4].try_into()?);
        i += 4;
        let input = bytes[i..i + input_len as usize].to_vec();
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
            input: input.into(),
            v: 0,
            r: 0,
            s: 0,
            chain_id: CHAIN_ID,
            tx_type: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::types::BEncodeDecode;
    use crate::server::CHAIN_ID;

    #[test]
    fn encode_decode() {
        let tx = TxED {
            hash: BEncodeDecode([1u8; 32].into()),
            nonce: 1,
            block_hash: BEncodeDecode([2u8; 32].into()),
            block_number: 2,
            transaction_index: 3,
            from: AddressED([3u8; 20].into()),
            to: Some(AddressED([4u8; 20].into())),
            value: 4,
            gas: 5,
            gas_price: 6,
            input: vec![7, 8, 9].into(),
            v: 0,
            r: 0,
            s: 0,
            chain_id: CHAIN_ID,
            tx_type: 0,
        };
        let encoded = tx.encode();
        let decoded = TxED::decode(encoded).unwrap();
        assert_eq!(tx, decoded);
    }

    #[test]
    fn serialize() {
        let tx = TxED {
            hash: BEncodeDecode([1u8; 32].into()),
            nonce: 1,
            block_hash: BEncodeDecode([2u8; 32].into()),
            block_number: 2,
            transaction_index: 3,
            from: AddressED([3u8; 20].into()),
            to: Some(AddressED([4u8; 20].into())),
            value: 4,
            gas: 5,
            gas_price: 6,
            input: vec![7, 8, 9].into(),
            v: 0,
            r: 0,
            s: 0,
            chain_id: CHAIN_ID,
            tx_type: 0,
        };
        let serialized = serde_json::to_string(&tx).unwrap();
        assert_eq!(
            serialized,
            "{\"hash\":\"0x0101010101010101010101010101010101010101010101010101010101010101\",\"nonce\":\"0x1\",\"blockHash\":\"0x0202020202020202020202020202020202020202020202020202020202020202\",\"blockNumber\":\"0x2\",\"transactionIndex\":\"0x3\",\"from\":\"0x0303030303030303030303030303030303030303\",\"to\":\"0x0404040404040404040404040404040404040404\",\"value\":\"0x4\",\"gas\":\"0x5\",\"gasPrice\":\"0x6\",\"input\":\"0x070809\",\"v\":\"0x0\",\"r\":\"0x0\",\"s\":\"0x0\"}"
        )
    }
}
