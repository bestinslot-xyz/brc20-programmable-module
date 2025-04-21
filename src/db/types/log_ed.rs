use std::error::Error;

use alloy_primitives::Log;
use serde::Serialize;

use crate::db::types::{AddressED, BytesED, Decode, Encode, B256ED, U64ED};

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct LogED {
    pub address: AddressED,
    pub topics: Vec<B256ED>,
    pub data: BytesED,
    #[serde(rename = "transactionIndex")]
    pub transaction_index: U64ED,
    #[serde(rename = "transactionHash")]
    pub transaction_hash: B256ED,
    #[serde(rename = "blockHash")]
    pub block_hash: B256ED,
    #[serde(rename = "blockNumber")]
    pub block_number: U64ED,
    #[serde(rename = "logIndex")]
    pub log_index: U64ED,
}

impl LogED {
    pub fn new_vec(
        logs: &Vec<Log>,
        mut log_index: u64,
        transaction_index: U64ED,
        transaction_hash: B256ED,
        block_hash: B256ED,
        block_number: U64ED,
    ) -> Vec<LogED> {
        let mut log_responses = Vec::new();
        for log in logs {
            log_responses.push(LogED {
                address: log.address.into(),
                topics: log.topics().iter().map(|topic| (*topic).into()).collect(),
                data: log.data.data.clone().into(),
                transaction_index: transaction_index.clone(),
                transaction_hash: transaction_hash.clone(),
                block_hash: block_hash.clone(),
                block_number: block_number.clone(),
                log_index: log_index.into(),
            });
            log_index += 1;
        }
        log_responses
    }
}

impl Encode for LogED {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.address.encode(buffer);
        self.topics.encode(buffer);
        self.data.encode(buffer);
        self.transaction_index.encode(buffer);
        self.transaction_hash.encode(buffer);
        self.block_hash.encode(buffer);
        self.block_number.encode(buffer);
        self.log_index.encode(buffer);
    }
}

impl Decode for LogED {
    fn decode(bytes: &[u8], offset: usize) -> Result<(Self, usize), Box<dyn Error>> {
        let (address, offset) = Decode::decode(bytes, offset)?;
        let (topics, offset) = Decode::decode(bytes, offset)?;
        let (data, offset) = Decode::decode(bytes, offset)?;
        let (transaction_index, offset) = Decode::decode(bytes, offset)?;
        let (transaction_hash, offset) = Decode::decode(bytes, offset)?;
        let (block_hash, offset) = Decode::decode(bytes, offset)?;
        let (block_number, offset) = Decode::decode(bytes, offset)?;
        let (log_index, offset) = Decode::decode(bytes, offset)?;

        Ok((
            LogED {
                address,
                topics,
                data,
                transaction_index,
                transaction_hash,
                block_hash,
                block_number,
                log_index,
            },
            offset,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_ed() {
        let log = Log::new(
            [1u8; 20].into(),
            vec![[2u8; 32].into()],
            [3u8; 32].to_vec().into(),
        )
        .unwrap();
        let transaction_index = 1u32;
        let transaction_hash: B256ED = [4u8; 32].into();
        let block_hash: B256ED = [5u8; 32].into();
        let block_number = 2u32;

        let log_response = LogED::new_vec(
            &vec![log],
            0u64.into(),
            transaction_index.into(),
            transaction_hash.clone(),
            block_hash.clone(),
            block_number.into(),
        );

        assert_eq!(log_response.len(), 1);
        assert_eq!(log_response[0].transaction_index, transaction_index.into());
        assert_eq!(log_response[0].transaction_hash, transaction_hash);
        assert_eq!(log_response[0].block_hash, block_hash);
        assert_eq!(log_response[0].block_number, block_number.into());
        assert_eq!(log_response[0].log_index, 0u64.into());
        assert_eq!(log_response[0].address, [1u8; 20].into());
        assert_eq!(log_response[0].topics, vec![[2u8; 32].into()]);
        assert_eq!(log_response[0].data, [3u8; 32].to_vec().into());

        let bytes = log_response[0].encode_vec();
        let decoded = LogED::decode_vec(&bytes).unwrap();
        assert_eq!(log_response[0], decoded);
        assert_eq!(log_response[0].address, decoded.address);
        assert_eq!(log_response[0].topics, decoded.topics);
        assert_eq!(log_response[0].data, decoded.data);
        assert_eq!(log_response[0].transaction_index, decoded.transaction_index);
        assert_eq!(log_response[0].transaction_hash, decoded.transaction_hash);
        assert_eq!(log_response[0].block_hash, decoded.block_hash);
        assert_eq!(log_response[0].block_number, decoded.block_number);
        assert_eq!(log_response[0].log_index, decoded.log_index);
    }

    #[test]
    fn test_log_ed_serialize() {
        let log = Log::new(
            [1u8; 20].into(),
            vec![[2u8; 32].into()],
            [3u8; 32].to_vec().into(),
        )
        .unwrap();
        let transaction_index = 1u32;
        let transaction_hash: B256ED = [4u8; 32].into();
        let block_hash: B256ED = [5u8; 32].into();
        let block_number = 2u32;

        let log_responses = LogED::new_vec(
            &vec![log],
            0u64.into(),
            transaction_index.into(),
            transaction_hash.clone(),
            block_hash.clone(),
            block_number.into(),
        );

        assert_eq!(log_responses.len(), 1);
        assert_eq!(log_responses[0].transaction_index, transaction_index.into());
        assert_eq!(log_responses[0].transaction_hash, transaction_hash);
        assert_eq!(log_responses[0].block_hash, block_hash);
        assert_eq!(log_responses[0].block_number, block_number.into());
    }

    #[test]
    fn test_log_response_empty() {
        let transaction_index = 1u32;
        let transaction_hash: B256ED = [4u8; 32].into();
        let block_hash: B256ED = [5u8; 32].into();
        let block_number = 2u32;

        let log_responses = LogED::new_vec(
            &vec![],
            0u64.into(),
            transaction_index.into(),
            transaction_hash.clone(),
            block_hash.clone(),
            block_number.into(),
        );

        assert_eq!(log_responses.len(), 0);
    }

    #[test]
    fn test_log_response_multiple() {
        let log1 = Log::new(
            [1u8; 20].into(),
            vec![[2u8; 32].into()],
            [3u8; 32].to_vec().into(),
        )
        .unwrap();
        let log2 = Log::new(
            [4u8; 20].into(),
            vec![[5u8; 32].into()],
            [6u8; 32].to_vec().into(),
        )
        .unwrap();
        let transaction_index = 1u32;
        let transaction_hash: B256ED = [7u8; 32].into();
        let block_hash: B256ED = [8u8; 32].into();
        let block_number = 2u32;

        let log_responses = LogED::new_vec(
            &vec![log1, log2],
            0u64.into(),
            transaction_index.into(),
            transaction_hash.clone(),
            block_hash.clone(),
            block_number.into(),
        );

        assert_eq!(log_responses.len(), 2);
    }
}
