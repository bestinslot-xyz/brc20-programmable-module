use std::error::Error;

use revm::primitives::{Address, Bytes, Log};
use serde::Serialize;
use serde_json::Map;

use crate::db::types::{AddressED, Decode, Encode, B256ED, U64ED};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogED {
    pub logs: Vec<Log>,
    pub log_index: u64,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct LogResponseED {
    pub address: AddressED,
    pub topics: Vec<B256ED>,
    #[serde(serialize_with = "bytes_hex")]
    pub data: Bytes,
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

fn bytes_hex<S>(bytes: &Bytes, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&format!("0x{:x}", bytes))
}

impl Serialize for LogED {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let logs: Vec<Map<String, serde_json::Value>> = self
            .logs
            .iter()
            .map(|log| {
                let mut map = Map::new();
                map.insert(
                    "address".to_string(),
                    serde_json::Value::String(format!("0x{:x}", log.address.0)),
                );
                map.insert(
                    "topics".to_string(),
                    serde_json::Value::Array(
                        log.topics()
                            .iter()
                            .map(|topic| serde_json::Value::String(format!("0x{:x}", topic)))
                            .collect(),
                    ),
                );
                map.insert(
                    "data".to_string(),
                    serde_json::Value::String(format!("{:x}", log.data.data)),
                );
                map
            })
            .collect();
        serde_json::Value::Array(logs.into_iter().map(serde_json::Value::Object).collect())
            .serialize(serializer)
    }
}

impl Encode for LogED {
    fn encode(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(self.log_index.to_be_bytes()));
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
        Ok(bytes)
    }
}

impl Decode for LogED {
    fn decode(bytes: Vec<u8>) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let mut logs = Vec::new();
        let mut i = 0;
        let log_index = u64::from_be_bytes(bytes[i..i + 8].try_into().unwrap());
        i += 8;
        let logs_len = u32::from_be_bytes(bytes[i..i + 4].try_into().unwrap()) as usize;
        i += 4;
        for _ in 0..logs_len {
            let address = Address::from_slice(&bytes[i..i + 20]);
            i += 20;

            let topics_len = u32::from_be_bytes(bytes[i..i + 4].try_into().unwrap());
            i += 4;

            let mut topics = Vec::new();
            for _ in 0..topics_len {
                let topic = B256ED::decode(bytes[i..i + 32].try_into().unwrap()).unwrap();
                topics.push(topic.0);
                i += 32;
            }

            let data_len = u32::from_be_bytes(bytes[i..i + 4].try_into().unwrap()) as usize;
            i += 4;

            let data = bytes[i..i + data_len].to_vec().try_into().unwrap();
            i += data_len;
            logs.push(Log::new(address, topics, data).unwrap());
        }
        Ok(LogED { logs, log_index })
    }
}

#[cfg(test)]
mod tests {
    use revm::primitives::{Address, Log, B256};

    use super::*;

    #[test]
    fn test_log_ed() {
        let log = Log::new(
            Address::from([1u8; 20]),
            vec![B256::from([2u8; 32])],
            [3u8; 32].to_vec().into(),
        )
        .unwrap();
        let log_ed = LogED {
            logs: vec![log],
            log_index: 0,
        };
        let bytes = LogED::encode(&log_ed).unwrap();
        assert_eq!(bytes.len(), 104);
        let decoded = LogED::decode(bytes).unwrap();
        assert_eq!(log_ed.logs, decoded.logs);
        assert_eq!(log_ed.log_index, decoded.log_index);
    }

    #[test]
    fn test_log_ed_serialize() {
        let log = Log::new(
            Address::from([1u8; 20]),
            vec![B256::from([2u8; 32])],
            [3u8; 32].to_vec().into(),
        )
        .unwrap();
        let log_ed = LogED {
            logs: vec![log],
            log_index: 0,
        };
        let serialized = serde_json::to_string(&log_ed).unwrap();
        assert_eq!(
            serialized,
            "[{\"address\":\"0x0101010101010101010101010101010101010101\",\"data\":\"0x0303030303030303030303030303030303030303030303030303030303030303\",\"topics\":[\"0x0202020202020202020202020202020202020202020202020202020202020202\"]}]"
        );
    }
}
