use serde::Serialize;

use crate::db::types::{
    uint_full_hex, AddressED, Decode, Encode, TxED, B2048ED, B256ED, U128ED, U64ED,
};

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct BlockResponseED {
    pub difficulty: U64ED,
    #[serde(rename = "gasLimit")]
    pub gas_limit: U64ED,
    #[serde(rename = "gasUsed")]
    pub gas_used: U64ED,
    pub hash: B256ED,
    #[serde(rename = "logsBloom")]
    pub logs_bloom: B2048ED,
    #[serde(serialize_with = "uint_full_hex")]
    pub nonce: U64ED,
    pub number: U64ED,
    pub timestamp: U64ED,
    #[serde(rename = "mineTimestamp")]
    pub mine_timestamp: U128ED,

    #[serde(rename = "transactions", skip_serializing_if = "Option::is_none")]
    pub transactions: Option<Vec<B256ED>>,

    #[serde(rename = "transactions", skip_serializing_if = "Option::is_none")]
    pub full_transactions: Option<Vec<TxED>>,

    // Always empty values
    #[serde(rename = "baseFeePerGas")]
    pub base_fee_per_gas: U64ED,

    #[serde(rename = "transactionsRoot")]
    pub transactions_root: B256ED,

    #[serde(rename = "uncles")]
    pub uncles: Vec<B256ED>,

    pub withdrawals: Vec<B256ED>,

    #[serde(rename = "withdrawalsRoot")]
    pub withdrawals_root: B256ED,

    #[serde(rename = "totalDifficulty")]
    pub total_difficulty: U64ED,

    #[serde(rename = "parentBeaconBlockRoot")]
    pub parent_beacon_block_root: B256ED,

    #[serde(rename = "parentHash")]
    pub parent_hash: B256ED,

    #[serde(rename = "receiptsRoot")]
    pub receipts_root: B256ED,

    #[serde(rename = "sha3Uncles")]
    pub sha3_uncles: B256ED,

    #[serde(rename = "size")]
    pub size: U64ED,

    #[serde(rename = "stateRoot")]
    pub state_root: B256ED,

    #[serde(rename = "miner")]
    pub miner: AddressED,

    #[serde(rename = "mixHash")]
    pub mix_hash: B256ED,

    #[serde(rename = "excessBlobGas")]
    pub excess_blob_gas: U64ED,

    #[serde(rename = "extraData")]
    pub extra_data: B256ED,

    #[serde(rename = "blobGasUsed")]
    pub blob_gas_used: U64ED,
}

impl BlockResponseED {
    pub fn new(
        difficulty: U64ED,
        gas_limit: U64ED,
        gas_used: U64ED,
        hash: B256ED,
        logs_bloom: B2048ED,
        nonce: U64ED,
        number: U64ED,
        timestamp: U64ED,
        mine_timestamp: U128ED,
        transactions: Vec<B256ED>,
        transactions_root: B256ED,
        total_difficulty: U64ED,
        parent_hash: B256ED,
        receipts_root: B256ED,
        size: U64ED,
    ) -> Self {
        Self {
            difficulty,
            gas_limit,
            gas_used,
            hash,
            logs_bloom,
            nonce,
            number,
            timestamp,
            mine_timestamp,
            transactions: Some(transactions),
            full_transactions: None,
            transactions_root,
            size,
            parent_hash,
            receipts_root,
            total_difficulty,
            base_fee_per_gas: 0u64.into(),
            uncles: Vec::new(),
            withdrawals: Vec::new(),
            withdrawals_root: [0u8; 32].into(),
            parent_beacon_block_root: [0u8; 32].into(),
            sha3_uncles: [0u8; 32].into(),
            state_root: [0u8; 32].into(),
            miner: [0u8; 20].into(),
            mix_hash: [0u8; 32].into(),
            excess_blob_gas: 0u64.into(),
            extra_data: [0u8; 32].into(),
            blob_gas_used: 0u64.into(),
        }
    }
}

impl Encode for BlockResponseED {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.difficulty.encode());
        bytes.extend_from_slice(&self.gas_limit.encode());
        bytes.extend_from_slice(&self.gas_used.encode());
        bytes.extend_from_slice(&self.hash.encode());
        bytes.extend_from_slice(&self.logs_bloom.encode());
        bytes.extend_from_slice(&self.nonce.encode());
        bytes.extend_from_slice(&self.number.encode());
        bytes.extend_from_slice(&self.timestamp.encode());
        bytes.extend_from_slice(&self.mine_timestamp.encode());
        let transactions = self.transactions.clone().unwrap_or(vec![]);
        let transactions_count = transactions.len() as u32;
        bytes.extend_from_slice(&transactions_count.to_be_bytes());
        for tx in &transactions {
            bytes.extend_from_slice(&tx.encode());
        }
        bytes.extend_from_slice(&self.transactions_root.encode());
        bytes.extend_from_slice(&self.total_difficulty.encode());
        bytes.extend_from_slice(&self.parent_hash.encode());
        bytes.extend_from_slice(&self.receipts_root.encode());
        bytes.extend_from_slice(&self.size.encode());
        bytes
    }
}

impl Decode for BlockResponseED {
    fn decode(bytes: Vec<u8>) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        let mut i = 0;
        let difficulty = U64ED::decode(bytes[i..i + 8].try_into()?)?;
        i += 8;
        let gas_limit = U64ED::decode(bytes[i..i + 8].try_into()?)?;
        i += 8;
        let gas_used = U64ED::decode(bytes[i..i + 8].try_into()?)?;
        i += 8;
        let hash = B256ED::decode(bytes[i..i + 32].to_vec())?;
        i += 32;
        let logs_bloom = B2048ED::decode(bytes[i..i + 256].to_vec())?;
        i += 256;
        let nonce = U64ED::decode(bytes[i..i + 8].try_into()?)?;
        i += 8;
        let number = U64ED::decode(bytes[i..i + 8].try_into()?)?;
        i += 8;
        let timestamp = U64ED::decode(bytes[i..i + 8].try_into()?)?;
        i += 8;
        let mine_timestamp = U128ED::decode(bytes[i..i + 16].to_vec())?;
        i += 16;
        let transactions_count = u32::from_be_bytes(bytes[i..i + 4].try_into()?);
        i += 4;
        let mut transactions = Vec::new();
        for _ in 0..transactions_count {
            let tx = B256ED::decode(bytes[i..i + 32].to_vec())?;
            transactions.push(tx);
            i += 32;
        }
        let transactions_root = B256ED::decode(bytes[i..i + 32].to_vec())?;
        i += 32;
        let total_difficulty = U64ED::decode(bytes[i..i + 8].try_into()?)?;
        i += 8;
        let parent_hash = B256ED::decode(bytes[i..i + 32].to_vec())?;
        i += 32;
        let receipts_root = B256ED::decode(bytes[i..i + 32].to_vec())?;
        i += 32;
        let size = U64ED::decode(bytes[i..i + 8].try_into()?)?;

        Ok(BlockResponseED::new(
            difficulty,
            gas_limit,
            gas_used,
            hash,
            logs_bloom,
            nonce,
            number,
            timestamp,
            mine_timestamp,
            transactions,
            transactions_root,
            total_difficulty,
            parent_hash,
            receipts_root,
            size,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_response_encode_decode() {
        let block = BlockResponseED::new(
            1u64.into(),
            2u64.into(),
            3u64.into(),
            [4u8; 32].into(),
            [5u8; 256].into(),
            6u64.into(),
            7u64.into(),
            8u64.into(),
            9u64.into(),
            vec![[10u8; 32].into(), [11u8; 32].into()],
            [12u8; 32].into(),
            13u64.into(),
            [14u8; 32].into(),
            [15u8; 32].into(),
            16u64.into(),
        );

        let encoded = block.encode();
        let decoded = BlockResponseED::decode(encoded).unwrap();

        assert_eq!(block, decoded);
    }

    #[test]
    fn test_block_response_serialize() {
        let block = BlockResponseED::new(
            1u64.into(),
            2u64.into(),
            3u64.into(),
            [4u8; 32].into(),
            [5u8; 256].into(),
            6u64.into(),
            7u64.into(),
            8u64.into(),
            9u64.into(),
            vec![[10u8; 32].into(), [11u8; 32].into()],
            [12u8; 32].into(),
            13u64.into(),
            [14u8; 32].into(),
            [15u8; 32].into(),
            16u64.into(),
        );

        let serialized = serde_json::to_string(&block).unwrap();
        assert_eq!(serialized, "{\"difficulty\":\"0x1\",\"gasLimit\":\"0x2\",\"gasUsed\":\"0x3\",\"hash\":\"0x0404040404040404040404040404040404040404040404040404040404040404\",\"logsBloom\":\"0x05050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505\",\"nonce\":\"0x0000000000000006\",\"number\":\"0x7\",\"timestamp\":\"0x8\",\"mineTimestamp\":\"0x9\",\"transactions\":[\"0x0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a\",\"0x0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b\"],\"baseFeePerGas\":\"0x0\",\"transactionsRoot\":\"0x0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c\",\"uncles\":[],\"withdrawals\":[],\"withdrawalsRoot\":\"0x0000000000000000000000000000000000000000000000000000000000000000\",\"totalDifficulty\":\"0xd\",\"parentBeaconBlockRoot\":\"0x0000000000000000000000000000000000000000000000000000000000000000\",\"parentHash\":\"0x0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e\",\"receiptsRoot\":\"0x0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f\",\"sha3Uncles\":\"0x0000000000000000000000000000000000000000000000000000000000000000\",\"size\":\"0x10\",\"stateRoot\":\"0x0000000000000000000000000000000000000000000000000000000000000000\",\"miner\":\"0x0000000000000000000000000000000000000000\",\"mixHash\":\"0x0000000000000000000000000000000000000000000000000000000000000000\",\"excessBlobGas\":\"0x0\",\"extraData\":\"0x0000000000000000000000000000000000000000000000000000000000000000\",\"blobGasUsed\":\"0x0\"}");
    }
}
