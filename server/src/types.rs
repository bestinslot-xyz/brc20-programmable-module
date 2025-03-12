use serde::Deserialize;

use revm::primitives::alloy_primitives::Bytes;
use revm::primitives::{
    keccak256, Address, B256,
};

#[derive(Deserialize, Clone)]
pub struct TxInfo {
    #[serde(deserialize_with = "deserialize_address")]
    pub from: Address,
    #[serde(deserialize_with = "deserialize_option_address")]
    pub to: Option<Address>,
    #[serde(deserialize_with = "deserialize_data")]
    pub data: Bytes,
}

fn deserialize_address<'de, D>(deserializer: D) -> Result<Address, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let s = if s.starts_with("0x") { &s[2..] } else { &s };
    return Ok(Address::from_slice(&hex::decode(s).unwrap()));
}

fn deserialize_option_address<'de, D>(deserializer: D) -> Result<Option<Address>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer);
    if s.is_err() {
        return Ok(None);
    }
    let s = s.unwrap();
    let s = if s.starts_with("0x") { &s[2..] } else { &s };
    let bytes = hex::decode(s);
    if bytes.is_err() {
        return Ok(None);
    }
    Ok(Some(Address::from_slice(&bytes.unwrap())))
}

fn deserialize_data<'de, D>(deserializer: D) -> Result<Bytes, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let s = if s.starts_with("0x") { &s[2..] } else { &s };
    let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
    Ok(Bytes::from(bytes))
}

pub fn get_tx_hash(txinfo: &TxInfo, nonce: &u64) -> B256 {
    let mut data = Vec::new();
    data.extend_from_slice(txinfo.from.as_slice());
    data.extend_from_slice(&nonce.to_be_bytes());
    if let Some(to) = txinfo.to {
        data.extend_from_slice(to.as_slice());
    } else {
        data.extend_from_slice(&[0; 20]);
    }
    data.extend_from_slice(&txinfo.data);
    keccak256(data)
}
