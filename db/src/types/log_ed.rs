use std::error::Error;

use revm::primitives::{Address, Log};
use serde::Serialize;
use serde_json::Map;

use super::{Decode, Encode, B256ED};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogED(pub Vec<Log>);

impl Serialize for LogED {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let logs: Vec<Map<String, serde_json::Value>> = self
            .0
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
                    serde_json::Value::String(format!("0x{:x}", log.data.data)),
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
        for log in self.0.iter() {
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
        while i < bytes.len() {
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
        Ok(LogED(logs))
    }
}

#[cfg(test)]
mod tests {
    use revm::primitives::{Address, Log, B256};

    use crate::types::{Decode, Encode, LogED};

    #[test]
    fn test_log_ed() {
        let log = Log::new(
            Address::from([1u8; 20]),
            vec![B256::from([2u8; 32])],
            [3u8; 32].to_vec().into(),
        )
        .unwrap();
        let log_ed = LogED(vec![log]);
        let bytes = LogED::encode(&log_ed).unwrap();
        assert_eq!(bytes.len(), 92);
        let decoded = LogED::decode(bytes).unwrap();
        assert_eq!(log_ed.0, decoded.0);
    }

    #[test]
    fn test_log_ed_serialize() {
        let log = Log::new(
            Address::from([1u8; 20]),
            vec![B256::from([2u8; 32])],
            [3u8; 32].to_vec().into(),
        )
        .unwrap();
        let log_ed = LogED(vec![log]);
        let serialized = serde_json::to_string(&log_ed).unwrap();
        assert_eq!(
            serialized,
            "[{\"address\":\"0x0101010101010101010101010101010101010101\",\"data\":\"0x0x0303030303030303030303030303030303030303030303030303030303030303\",\"topics\":[\"0x0202020202020202020202020202020202020202020202020202020202020202\"]}]"
        );
    }
}
