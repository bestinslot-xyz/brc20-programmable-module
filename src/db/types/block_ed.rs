use std::error::Error;

use either::Either::{self, Left, Right};
use serde::{Deserialize, Serialize};

use crate::db::types::{
    uint_full_hex, AddressED, Decode, Encode, TxED, B2048ED, B256ED, U128ED, U64ED,
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
/// Represents a block response from BRC2.0 with all the fields required by the API.
///
/// Refer to [Ethereum JSON-RPC documentation on eth_getBlockByHash](https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_getblockbyhash) for details
pub struct BlockResponseED {
    /// The difficulty of the block
    pub difficulty: U64ED,
    #[serde(rename = "gasLimit")]
    /// The gas limit of the block
    pub gas_limit: U64ED,
    #[serde(rename = "gasUsed")]
    /// The gas used by the block
    pub gas_used: U64ED,
    /// Block hash
    pub hash: B256ED,
    #[serde(rename = "logsBloom")]
    /// The bloom filter for the logs in the block
    pub logs_bloom: B2048ED,
    #[serde(serialize_with = "uint_full_hex")]
    /// The nonce of the block
    pub nonce: U64ED,
    /// The block number
    pub number: U64ED,
    /// The timestamp of the block
    pub timestamp: U64ED,
    /// Specific to BRC2.0, time it took to index the block
    #[serde(rename = "mineTimestamp")]
    pub mine_timestamp: U128ED,

    #[serde(
        rename = "transactions",
        serialize_with = "tx_serialize",
        deserialize_with = "tx_deserialize"
    )]
    /// The transactions in the block
    ///
    /// Can be a list of hashes or a list of transactions, depending on the call's is_full flag
    pub transactions: Either<Vec<B256ED>, Vec<TxED>>,

    #[serde(rename = "baseFeePerGas")]
    /// The base fee per gas for the block, 0 for BRC2.0
    pub base_fee_per_gas: U64ED,

    #[serde(rename = "transactionsRoot")]
    /// The root hash of the transactions in the block
    pub transactions_root: B256ED,

    #[serde(rename = "uncles")]
    /// The uncles of the block, empty for BRC2.0
    pub uncles: Vec<B256ED>,

    /// The withdrawals of the block, empty for BRC2.0
    pub withdrawals: Vec<B256ED>,

    #[serde(rename = "withdrawalsRoot")]
    /// The root hash of the withdrawals in the block, empty for BRC2.0
    pub withdrawals_root: B256ED,

    #[serde(rename = "totalDifficulty")]
    /// The total difficulty of the block, 0 for BRC2.0
    pub total_difficulty: U64ED,

    #[serde(rename = "parentBeaconBlockRoot")]
    /// The parent beacon block root, empty for BRC2.0
    pub parent_beacon_block_root: B256ED,

    #[serde(rename = "parentHash")]
    /// The parent hash of the block
    pub parent_hash: B256ED,

    #[serde(rename = "receiptsRoot")]
    /// The root hash of the receipts in the block
    pub receipts_root: B256ED,

    #[serde(rename = "sha3Uncles")]
    /// The sha3 uncles of the block, empty for BRC2.0
    pub sha3_uncles: B256ED,

    #[serde(rename = "size")]
    /// The size of the block, not recorded for BRC2.0
    pub size: U64ED,

    #[serde(rename = "stateRoot")]
    /// The root hash of the state in the block, not recorded for BRC2.0
    pub state_root: B256ED,

    #[serde(rename = "miner")]
    /// The miner of the block, not recorded for BRC2.0
    pub miner: AddressED,

    #[serde(rename = "mixHash")]
    /// The mix hash of the block, not recorded for BRC2.0
    pub mix_hash: B256ED,

    #[serde(rename = "excessBlobGas")]
    /// The excess blob gas of the block, not recorded for BRC2.0, as it does not support blob transactions
    pub excess_blob_gas: U64ED,

    #[serde(rename = "extraData")]
    /// The extra data of the block, not recorded for BRC2.0
    pub extra_data: B256ED,

    #[serde(rename = "blobGasUsed")]
    /// The blob gas used of the block, not recorded for BRC2.0, as it does not support blob transactions
    pub blob_gas_used: U64ED,
}

fn tx_serialize<S>(tx: &Either<Vec<B256ED>, Vec<TxED>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match tx {
        Left(hashes) => hashes.serialize(serializer),
        Right(txs) => txs.serialize(serializer),
    }
}

fn tx_deserialize<'de, D>(deserializer: D) -> Result<Either<Vec<B256ED>, Vec<TxED>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let values = serde_json::Value::deserialize(deserializer)?;
    if !values.is_array() {
        return Err(serde::de::Error::custom("Expected an array"));
    } else {
        if !values.get(0).is_some_and(|v| v.is_string()) {
            let transactions: Vec<TxED> =
                serde_json::from_value(values).map_err(serde::de::Error::custom)?;
            return Ok(Right(transactions));
        } else {
            let hashes: Vec<B256ED> =
                serde_json::from_value(values).map_err(serde::de::Error::custom)?;
            return Ok(Left(hashes));
        }
    }
}

impl BlockResponseED {
    // This is returned by the API, so doesn't need to be public
    pub(crate) fn new(
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
            transactions: Either::Left(transactions),
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
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.difficulty.encode(buffer);
        self.gas_limit.encode(buffer);
        self.gas_used.encode(buffer);
        self.hash.encode(buffer);
        self.logs_bloom.encode(buffer);
        self.nonce.encode(buffer);
        self.number.encode(buffer);
        self.timestamp.encode(buffer);
        self.mine_timestamp.encode(buffer);
        let Left(transactions) = &self.transactions else {
            // This should never happen, and it would be an implementation error, just in case
            panic!("transactions should be a list of hashes");
        };
        transactions.encode(buffer);
        self.transactions_root.encode(buffer);
        self.total_difficulty.encode(buffer);
        self.parent_hash.encode(buffer);
        self.receipts_root.encode(buffer);
        self.size.encode(buffer);
    }
}

impl Decode for BlockResponseED {
    fn decode(bytes: &[u8], offset: usize) -> Result<(Self, usize), Box<dyn Error>> {
        let (difficulty, offset) = Decode::decode(bytes, offset)?;
        let (gas_limit, offset) = Decode::decode(bytes, offset)?;
        let (gas_used, offset) = Decode::decode(bytes, offset)?;
        let (hash, offset) = Decode::decode(bytes, offset)?;
        let (logs_bloom, offset) = Decode::decode(bytes, offset)?;
        let (nonce, offset) = Decode::decode(bytes, offset)?;
        let (number, offset) = Decode::decode(bytes, offset)?;
        let (timestamp, offset) = Decode::decode(bytes, offset)?;
        let (mine_timestamp, offset) = Decode::decode(bytes, offset)?;
        let (transactions, offset) = Decode::decode(bytes, offset)?;
        let (transactions_root, offset) = Decode::decode(bytes, offset)?;
        let (total_difficulty, offset) = Decode::decode(bytes, offset)?;
        let (parent_hash, offset) = Decode::decode(bytes, offset)?;
        let (receipts_root, offset) = Decode::decode(bytes, offset)?;
        let (size, offset) = Decode::decode(bytes, offset)?;

        Ok((
            BlockResponseED::new(
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
            ),
            offset,
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

        let encoded = block.encode_vec();
        let decoded = BlockResponseED::decode_vec(&encoded).unwrap();

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

        let deserialized: BlockResponseED = serde_json::from_str(&serialized).unwrap();
        assert_eq!(block, deserialized);
    }

    #[test]
    fn test_block_response_serde_full_txes() {
        let mut block = BlockResponseED::new(
            1u64.into(),
            2u64.into(),
            3u64.into(),
            [4u8; 32].into(),
            [5u8; 256].into(),
            6u64.into(),
            7u64.into(),
            8u64.into(),
            9u64.into(),
            vec![],
            [12u8; 32].into(),
            13u64.into(),
            [14u8; 32].into(),
            [15u8; 32].into(),
            16u64.into(),
        );

        block.transactions = Either::Right(vec![TxED::new(
            [17u8; 32].into(),
            18u64.into(),
            [19u8; 32].into(),
            20u64.into(),
            21u64.into(),
            [22u8; 20].into(),
            Some([23u8; 20].into()),
            24u64.into(),
            vec![25u8].into(),
            None,
        )]);

        let serialized = serde_json::to_string(&block).unwrap();
        assert_eq!(serialized, "{\"difficulty\":\"0x1\",\"gasLimit\":\"0x2\",\"gasUsed\":\"0x3\",\"hash\":\"0x0404040404040404040404040404040404040404040404040404040404040404\",\"logsBloom\":\"0x05050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505050505\",\"nonce\":\"0x0000000000000006\",\"number\":\"0x7\",\"timestamp\":\"0x8\",\"mineTimestamp\":\"0x9\",\"transactions\":[{\"hash\":\"0x1111111111111111111111111111111111111111111111111111111111111111\",\"nonce\":\"0x12\",\"blockHash\":\"0x1313131313131313131313131313131313131313131313131313131313131313\",\"blockNumber\":\"0x14\",\"transactionIndex\":\"0x15\",\"from\":\"0x1616161616161616161616161616161616161616\",\"to\":\"0x1717171717171717171717171717171717171717\",\"value\":\"0x0\",\"gas\":\"0x18\",\"gasPrice\":\"0x0\",\"input\":\"0x19\",\"v\":\"0x0\",\"r\":\"0x0\",\"s\":\"0x0\",\"chainId\":\"0x425243323073\",\"type\":0}],\"baseFeePerGas\":\"0x0\",\"transactionsRoot\":\"0x0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c\",\"uncles\":[],\"withdrawals\":[],\"withdrawalsRoot\":\"0x0000000000000000000000000000000000000000000000000000000000000000\",\"totalDifficulty\":\"0xd\",\"parentBeaconBlockRoot\":\"0x0000000000000000000000000000000000000000000000000000000000000000\",\"parentHash\":\"0x0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e\",\"receiptsRoot\":\"0x0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f\",\"sha3Uncles\":\"0x0000000000000000000000000000000000000000000000000000000000000000\",\"size\":\"0x10\",\"stateRoot\":\"0x0000000000000000000000000000000000000000000000000000000000000000\",\"miner\":\"0x0000000000000000000000000000000000000000\",\"mixHash\":\"0x0000000000000000000000000000000000000000000000000000000000000000\",\"excessBlobGas\":\"0x0\",\"extraData\":\"0x0000000000000000000000000000000000000000000000000000000000000000\",\"blobGasUsed\":\"0x0\"}");

        let deserialized: BlockResponseED = serde_json::from_str(&serialized).unwrap();
        assert_eq!(block, deserialized);
    }
}
