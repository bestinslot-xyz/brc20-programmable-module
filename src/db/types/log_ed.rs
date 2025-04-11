use std::error::Error;

use alloy_primitives::{Address, Log};
use serde::Serialize;

use crate::db::types::{AddressED, BytesED, Decode, Encode, B256ED, U64ED};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogED {
    pub logs: Vec<Log>,
    pub log_index: U64ED,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct LogResponse {
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

impl LogResponse {
    pub fn new_vec(
        log: &LogED,
        transaction_index: U64ED,
        transaction_hash: B256ED,
        block_hash: B256ED,
        block_number: U64ED,
    ) -> Vec<LogResponse> {
        let mut log_index: u64 = log.log_index.clone().into();
        let mut log_responses = Vec::new();
        for log in &log.logs {
            log_responses.push(LogResponse {
                address: AddressED(log.address),
                topics: log
                    .topics()
                    .iter()
                    .map(|topic| B256ED::from_b256(*topic))
                    .collect(),
                data: BytesED(log.data.data.clone()),
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
    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(self.log_index.encode()));
        bytes.extend_from_slice(&(self.logs.len() as u32).to_be_bytes());
        for log in self.logs.iter() {
            bytes.extend_from_slice(&log.address.0.to_vec());
            bytes.extend_from_slice(&(log.topics().len() as u32).to_be_bytes());
            for topic in log.topics().iter() {
                bytes.extend_from_slice(&topic.0.to_vec());
            }
            bytes.extend_from_slice(&(log.data.data.len() as u32).to_be_bytes());
            bytes.extend_from_slice(&log.data.data);
        }
        bytes
    }
}

impl Decode for LogED {
    fn decode(bytes: Vec<u8>) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let mut logs = Vec::new();
        let mut i = 0;
        let log_index = U64ED::decode(bytes[i..i + 8].try_into()?)?;
        i += 8;
        let logs_len = u32::from_be_bytes(bytes[i..i + 4].try_into()?) as usize;
        i += 4;
        for _ in 0..logs_len {
            let address = Address::from_slice(&bytes[i..i + 20]);
            i += 20;

            let topics_len = u32::from_be_bytes(bytes[i..i + 4].try_into()?);
            i += 4;

            let mut topics = Vec::new();
            for _ in 0..topics_len {
                let topic = B256ED::decode(bytes[i..i + 32].try_into()?)?;
                topics.push(topic.0);
                i += 32;
            }

            let data_len = u32::from_be_bytes(bytes[i..i + 4].try_into()?) as usize;
            i += 4;

            let data = bytes[i..i + data_len].to_vec().try_into()?;
            i += data_len;
            logs.push(Log::new_unchecked(address, topics, data));
        }
        Ok(LogED { logs, log_index })
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
        let log_ed = LogED {
            logs: vec![log],
            log_index: 0.into(),
        };
        let bytes = log_ed.encode();
        assert_eq!(bytes.len(), 104);
        let decoded = LogED::decode(bytes).unwrap();
        assert_eq!(log_ed.logs, decoded.logs);
        assert_eq!(log_ed.log_index, decoded.log_index);
    }

    #[test]
    fn test_log_response() {
        let log = Log::new(
            [1u8; 20].into(),
            vec![[2u8; 32].into()],
            [3u8; 32].to_vec().into(),
        )
        .unwrap();
        let log_ed = LogED {
            logs: vec![log],
            log_index: 0.into(),
        };
        let transaction_index = 1;
        let transaction_hash = B256ED::from_b256([4u8; 32].into());
        let block_hash = B256ED::from_b256([5u8; 32].into());
        let block_number = 2;

        let log_responses = LogResponse::new_vec(
            &log_ed,
            transaction_index.into(),
            transaction_hash.clone(),
            block_hash.clone(),
            block_number.into(),
        );

        assert_eq!(log_responses.len(), 1);
        assert_eq!(
            log_responses[0].transaction_index,
            U64ED::from(transaction_index)
        );
        assert_eq!(log_responses[0].transaction_hash, transaction_hash);
        assert_eq!(log_responses[0].block_hash, block_hash);
        assert_eq!(log_responses[0].block_number, U64ED::from(block_number));
    }

    #[test]
    fn test_log_response_empty() {
        let log = LogED {
            logs: vec![],
            log_index: 0.into(),
        };
        let transaction_index = 1;
        let transaction_hash = B256ED::from_b256([4u8; 32].into());
        let block_hash = B256ED::from_b256([5u8; 32].into());
        let block_number = 2;

        let log_responses = LogResponse::new_vec(
            &log,
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
        let log_ed = LogED {
            logs: vec![log1, log2],
            log_index: 0.into(),
        };
        let transaction_index = 1;
        let transaction_hash = B256ED::from_b256([7u8; 32].into());
        let block_hash = B256ED::from_b256([8u8; 32].into());
        let block_number = 2;

        let log_responses = LogResponse::new_vec(
            &log_ed,
            transaction_index.into(),
            transaction_hash.clone(),
            block_hash.clone(),
            block_number.into(),
        );

        assert_eq!(log_responses.len(), 2);
    }
}
