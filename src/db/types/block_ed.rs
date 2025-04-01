use revm::primitives::{Address, FixedBytes};
use serde::Serialize;
use serde_hex::{CompactPfx, SerHex, StrictPfx};

use super::TxED;
use crate::db::types::{AddressED, BEncodeDecode, Decode, Encode, B2048ED, B256ED, U128ED};

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct BlockResponseED {
    #[serde(with = "SerHex::<CompactPfx>")]
    pub difficulty: u64,
    #[serde(with = "SerHex::<CompactPfx>", rename = "gasLimit")]
    pub gas_limit: u64,
    #[serde(with = "SerHex::<CompactPfx>", rename = "gasUsed")]
    pub gas_used: u64,
    pub hash: B256ED,
    #[serde(rename = "logsBloom")]
    pub logs_bloom: B2048ED,
    #[serde(with = "SerHex::<StrictPfx>")]
    pub nonce: u64,
    #[serde(with = "SerHex::<CompactPfx>")]
    pub number: u64,
    #[serde(with = "SerHex::<CompactPfx>")]
    pub timestamp: u64,
    #[serde(rename = "mineTimestamp")]
    pub mine_timestamp: U128ED,

    #[serde(rename = "transactions", skip_serializing_if = "Option::is_none")]
    pub transactions: Option<Vec<B256ED>>,

    #[serde(rename = "transactions", skip_serializing_if = "Option::is_none")]
    pub full_transactions: Option<Vec<TxED>>,

    // Always empty values
    #[serde(with = "SerHex::<CompactPfx>", rename = "baseFeePerGas")]
    pub base_fee_per_gas: u64,

    #[serde(rename = "transactionsRoot")]
    pub transactions_root: B256ED,

    #[serde(rename = "uncles")]
    pub uncles: Vec<B256ED>,

    pub withdrawals: Vec<B256ED>,

    #[serde(rename = "withdrawalsRoot")]
    pub withdrawals_root: B256ED,

    #[serde(rename = "totalDifficulty", with = "SerHex::<CompactPfx>")]
    pub total_difficulty: u64,

    #[serde(rename = "parentBeaconBlockRoot")]
    pub parent_beacon_block_root: B256ED,

    #[serde(rename = "parentHash")]
    pub parent_hash: B256ED,

    #[serde(rename = "receiptsRoot")]
    pub receipts_root: B256ED,

    #[serde(rename = "sha3Uncles")]
    pub sha3_uncles: B256ED,

    #[serde(rename = "size")]
    pub size: u64,

    #[serde(rename = "stateRoot")]
    pub state_root: B256ED,

    #[serde(rename = "miner")]
    pub miner: AddressED,

    #[serde(rename = "mixHash")]
    pub mix_hash: B256ED,

    #[serde(rename = "excessBlobGas", with = "SerHex::<CompactPfx>")]
    pub excess_blob_gas: u64,

    #[serde(rename = "extraData")]
    pub extra_data: B256ED,

    #[serde(rename = "blobGasUsed", with = "SerHex::<CompactPfx>")]
    pub blob_gas_used: u64,
}

impl BlockResponseED {
    pub fn new(
        difficulty: u64,
        gas_limit: u64,
        gas_used: u64,
        hash: B256ED,
        logs_bloom: B2048ED,
        nonce: u64,
        number: u64,
        timestamp: u64,
        mine_timestamp: U128ED,
        transactions: Vec<B256ED>,
        transactions_root: B256ED,
        total_difficulty: u64,
        parent_hash: B256ED,
        receipts_root: B256ED,
        size: u64,
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
            base_fee_per_gas: 0,
            uncles: Vec::new(),
            withdrawals: Vec::new(),
            withdrawals_root: BEncodeDecode(FixedBytes([0u8; 32])),
            parent_beacon_block_root: BEncodeDecode(FixedBytes([0u8; 32])),
            sha3_uncles: BEncodeDecode(FixedBytes([0u8; 32])),
            state_root: BEncodeDecode(FixedBytes([0u8; 32])),
            miner: AddressED(Address::new([0u8; 20])),
            mix_hash: BEncodeDecode(FixedBytes([0u8; 32])),
            excess_blob_gas: 0,
            extra_data: BEncodeDecode(FixedBytes([0u8; 32])),
            blob_gas_used: 0,
        }
    }
}

impl Encode for BlockResponseED {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.difficulty.to_be_bytes());
        bytes.extend_from_slice(&self.gas_limit.to_be_bytes());
        bytes.extend_from_slice(&self.gas_used.to_be_bytes());
        bytes.extend_from_slice(&self.hash.encode());
        bytes.extend_from_slice(&self.logs_bloom.encode());
        bytes.extend_from_slice(&self.nonce.to_be_bytes());
        bytes.extend_from_slice(&self.number.to_be_bytes());
        bytes.extend_from_slice(&self.timestamp.to_be_bytes());
        bytes.extend_from_slice(&self.mine_timestamp.encode());
        let transactions = self.transactions.clone().unwrap_or(vec![]);
        let transactions_count = transactions.len() as u64;
        bytes.extend_from_slice(&transactions_count.to_be_bytes());
        for tx in &transactions {
            bytes.extend_from_slice(&tx.encode());
        }
        bytes.extend_from_slice(&self.transactions_root.encode());
        bytes.extend_from_slice(&self.total_difficulty.to_be_bytes());
        bytes.extend_from_slice(&self.parent_hash.encode());
        bytes.extend_from_slice(&self.receipts_root.encode());
        bytes.extend_from_slice(&self.size.to_be_bytes());
        bytes
    }
}

impl Decode for BlockResponseED {
    fn decode(bytes: Vec<u8>) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        let mut i = 0;
        let difficulty = u64::from_be_bytes(bytes[i..i + 8].try_into()?);
        i += 8;
        let gas_limit = u64::from_be_bytes(bytes[i..i + 8].try_into()?);
        i += 8;
        let gas_used = u64::from_be_bytes(bytes[i..i + 8].try_into()?);
        i += 8;
        let hash = B256ED::decode(bytes[i..i + 32].to_vec())?;
        i += 32;
        let logs_bloom = B2048ED::decode(bytes[i..i + 256].to_vec())?;
        i += 256;
        let nonce = u64::from_be_bytes(bytes[i..i + 8].try_into()?);
        i += 8;
        let number = u64::from_be_bytes(bytes[i..i + 8].try_into()?);
        i += 8;
        let timestamp = u64::from_be_bytes(bytes[i..i + 8].try_into()?);
        i += 8;
        let mine_timestamp = U128ED::decode(bytes[i..i + 16].to_vec())?;
        i += 16;
        let transactions_count = u64::from_be_bytes(bytes[i..i + 8].try_into()?);
        i += 8;
        let mut transactions = Vec::new();
        for _ in 0..transactions_count {
            let tx = B256ED::decode(bytes[i..i + 32].to_vec())?;
            transactions.push(tx);
            i += 32;
        }
        let transactions_root = B256ED::decode(bytes[i..i + 32].to_vec())?;
        i += 32;
        let total_difficulty = u64::from_be_bytes(bytes[i..i + 8].try_into()?);
        i += 8;
        let parent_hash = B256ED::decode(bytes[i..i + 32].to_vec())?;
        i += 32;
        let receipts_root = B256ED::decode(bytes[i..i + 32].to_vec())?;
        i += 32;
        let size = u64::from_be_bytes(bytes[i..i + 8].try_into()?);

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
            1,
            2,
            3,
            BEncodeDecode(FixedBytes([4u8; 32])),
            BEncodeDecode(FixedBytes([5u8; 256])),
            6,
            7,
            8,
            U128ED::from_u128(9),
            vec![
                BEncodeDecode(FixedBytes([10u8; 32])),
                BEncodeDecode(FixedBytes([11u8; 32])),
            ],
            BEncodeDecode(FixedBytes([12u8; 32])),
            13,
            BEncodeDecode(FixedBytes([14u8; 32])),
            BEncodeDecode(FixedBytes([15u8; 32])),
            16,
        );

        let encoded = block.encode();
        let decoded = BlockResponseED::decode(encoded).unwrap();

        assert_eq!(block, decoded);
    }
}
