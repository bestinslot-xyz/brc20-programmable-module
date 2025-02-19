use serde::Serialize;
use serde_hex::{CompactPfx, StrictPfx, SerHex};

use super::{AddressED, B2048ED, B256ED, U128ED};

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

    pub transactions: Vec<B256ED>,

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
