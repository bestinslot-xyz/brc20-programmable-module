use std::error::Error;

use alloy_primitives::{logs_bloom, Address, Bytes, B256};
use revm::context::result::ExecutionResult;
use serde::Serialize;
use serde_hex::{CompactPfx, SerHex};

use super::U64ED;
use crate::db::types::{AddressED, Decode, Encode, LogED, LogResponse, B2048ED, B256ED};

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct TxReceiptED {
    #[serde(serialize_with = "one_or_zero")]
    pub status: u8,
    #[serde(rename = "txResult")]
    pub transaction_result: String,
    #[serde(rename = "reason")]
    pub reason: String,
    #[serde(skip_serializing)]
    pub logs: LogED,
    #[serde(rename = "logs")]
    pub log_responses: Vec<LogResponse>,
    #[serde(rename = "gasUsed")]
    pub gas_used: U64ED,
    pub from: AddressED,
    pub to: Option<AddressED>,
    #[serde(rename = "contractAddress")]
    pub contract_address: Option<AddressED>,
    #[serde(rename = "logsBloom")]
    pub logs_bloom: B2048ED,
    #[serde(rename = "blockHash")]
    pub hash: B256ED,
    #[serde(rename = "blockNumber")]
    pub block_number: U64ED,
    #[serde(rename = "blockTimestamp")]
    pub block_timestamp: U64ED,
    #[serde(rename = "transactionHash")]
    pub transaction_hash: B256ED,
    #[serde(rename = "transactionIndex")]
    pub transaction_index: U64ED,
    #[serde(rename = "cumulativeGasUsed")]
    pub cumulative_gas_used: U64ED,
    #[serde(rename = "effectiveGasPrice")]
    pub effective_gas_price: U64ED,
    #[serde(rename = "type", with = "SerHex::<CompactPfx>")]
    pub transaction_type: u8,
    pub nonce: U64ED,
    #[serde(rename = "resultBytes", serialize_with = "bytes")]
    pub result_bytes: Option<Bytes>,
}

fn bytes<S>(bytes: &Option<Bytes>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match bytes {
        Some(bytes) => serializer.serialize_str(&format!("0x{}", hex::encode(bytes))),
        None => serializer.serialize_str("0x"),
    }
}

fn one_or_zero<S>(status: &u8, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if *status == 1 {
        serializer.serialize_str("0x1")
    } else {
        serializer.serialize_str("0x0")
    }
}

impl TxReceiptED {
    pub fn new(
        block_hash: B256,
        block_number: U64ED,
        block_timestamp: U64ED,
        contract_address: Option<Address>,
        from: Address,
        to: Option<Address>,
        tx_hash: B256,
        tx_idx: U64ED,
        output: &ExecutionResult,
        cumulative_gas_used: U64ED,
        nonce: U64ED,
        start_log_index: U64ED,
        r#type: String,
        reason: String,
        output_bytes: Option<&Bytes>,
    ) -> Result<Self, Box<dyn Error>> {
        let logs = LogED {
            logs: output.logs().to_vec(),
            log_index: start_log_index,
        };
        let logs_bloom = B2048ED::decode(logs_bloom(output.logs()).to_vec())?;
        Ok(TxReceiptED {
            status: output.is_success() as u8,
            transaction_result: r#type,
            reason,
            logs: logs.clone(),
            log_responses: LogResponse::new_vec(
                &logs,
                tx_idx.clone(),
                tx_hash.into(),
                block_hash.into(),
                block_number.clone(),
            ),
            gas_used: output.gas_used().into(),
            from: from.into(),
            to: to.map(Into::<AddressED>::into),
            contract_address: contract_address.map(Into::<AddressED>::into),
            logs_bloom,
            hash: block_hash.into(),
            block_number: block_number.clone(),
            block_timestamp: block_timestamp,
            transaction_hash: tx_hash.into(),
            transaction_index: tx_idx.clone(),
            cumulative_gas_used,
            nonce,
            result_bytes: output_bytes.cloned(),
            effective_gas_price: 0u64.into(),
            transaction_type: 0,
        })
    }
}

impl Encode for TxReceiptED {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.status);

        let r#type_bytes = self.transaction_result.as_bytes();
        bytes.extend_from_slice(&(r#type_bytes.len() as u32).to_be_bytes());
        bytes.extend_from_slice(r#type_bytes);

        let reason_bytes = self.reason.as_bytes();
        bytes.extend_from_slice(&(reason_bytes.len() as u32).to_be_bytes());
        bytes.extend_from_slice(reason_bytes);

        let logs_bytes = self.logs.encode();

        bytes.extend_from_slice(&(logs_bytes.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&logs_bytes);

        bytes.extend_from_slice(&self.gas_used.encode());
        bytes.extend_from_slice(&self.from.encode());
        bytes.extend_from_slice(&self.to.as_ref().unwrap_or(&Address::ZERO.into()).encode());

        bytes.extend_from_slice(
            &self
                .contract_address
                .as_ref()
                .unwrap_or(&Address::ZERO.into())
                .encode(),
        );

        bytes.extend_from_slice(&self.logs_bloom.encode());
        bytes.extend_from_slice(&self.hash.encode());
        bytes.extend_from_slice(&self.block_number.encode());
        bytes.extend_from_slice(&self.block_timestamp.encode());
        bytes.extend_from_slice(&self.transaction_hash.encode());
        bytes.extend_from_slice(&self.transaction_index.encode());
        bytes.extend_from_slice(&self.cumulative_gas_used.encode());
        bytes.extend_from_slice(&self.nonce.encode());

        let output_bytes = self.result_bytes.as_ref();
        if let Some(output_bytes) = output_bytes {
            bytes.extend_from_slice(&(output_bytes.len() as u32).to_be_bytes());
            bytes.extend_from_slice(output_bytes);
        } else {
            bytes.extend_from_slice(&(0u32).to_be_bytes());
        }
        bytes
    }
}

impl Decode for TxReceiptED {
    fn decode(bytes: Vec<u8>) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        let status = bytes[0];
        let mut i = 1;

        let r#type_len = u32::from_be_bytes(bytes[i..i + 4].try_into()?) as usize;
        i += 4;
        let r#type = String::from_utf8(bytes[i..i + r#type_len].to_vec())?;
        i += r#type_len;

        let reason_len = u32::from_be_bytes(bytes[i..i + 4].try_into()?) as usize;
        i += 4;
        let reason = String::from_utf8(bytes[i..i + reason_len].to_vec())?;
        i += reason_len;

        let logs_len = u32::from_be_bytes(bytes[i..i + 4].try_into()?) as usize;
        i += 4;

        let logs = LogED::decode(bytes[i..i + logs_len].to_vec())?;
        i += logs_len;

        let gas_used = U64ED::decode(bytes[i..i + 8].try_into()?)?;
        i += 8;
        let from = AddressED::decode(bytes[i..i + 20].to_vec())?;
        i += 20;
        let to = AddressED::decode(bytes[i..i + 20].to_vec())?;
        i += 20;
        let contract_address = AddressED::decode(bytes[i..i + 20].to_vec())?;
        i += 20;
        let logs_bloom = B2048ED::decode(bytes[i..i + 256].to_vec())?;
        i += 256;
        let block_hash = B256ED::decode(bytes[i..i + 32].to_vec())?;
        i += 32;
        let block_number = U64ED::decode(bytes[i..i + 8].try_into()?)?;
        i += 8;
        let block_timestamp = U64ED::decode(bytes[i..i + 8].try_into()?)?;
        i += 8;
        let transaction_hash = B256ED::decode(bytes[i..i + 32].to_vec())?;
        i += 32;
        let transaction_index = U64ED::decode(bytes[i..i + 8].try_into()?)?;
        i += 8;
        let cumulative_gas_used = U64ED::decode(bytes[i..i + 8].try_into()?)?;
        i += 8;
        let nonce = U64ED::decode(bytes[i..i + 8].try_into()?)?;
        i += 8;
        let output_bytes_len = u32::from_be_bytes(bytes[i..i + 4].try_into()?) as usize;
        i += 4;
        let result_bytes = if output_bytes_len == 0 {
            None
        } else {
            Some(bytes[i..i + output_bytes_len].to_vec().into())
        };
        Ok(TxReceiptED {
            status,
            transaction_result: r#type,
            reason,
            logs: logs.clone(),
            log_responses: LogResponse::new_vec(
                &logs,
                transaction_index.clone(),
                transaction_hash.clone(),
                block_hash.clone(),
                block_number.clone(),
            ),
            gas_used,
            from,
            to: if to.is_zero() { None } else { Some(to) },
            contract_address: if contract_address.is_zero() {
                None
            } else {
                Some(contract_address)
            },
            logs_bloom,
            hash: block_hash,
            block_number,
            block_timestamp,
            transaction_hash,
            transaction_index,
            cumulative_gas_used,
            effective_gas_price: 0u64.into(),
            transaction_type: 0,
            nonce,
            result_bytes: result_bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::Log;

    use super::*;

    #[test]
    fn test_tx_receipt_ed() {
        let logs = LogED {
            logs: vec![Log::new(
                [1u8; 20].into(),
                vec![[2u8; 32].into()],
                vec![3u8; 32].into(),
            )
            .unwrap()],
            log_index: 0u64.into(),
        };
        let tx_receipt_ed = TxReceiptED {
            status: 4,
            transaction_result: "type".to_string(),
            reason: "reason".to_string(),
            logs: logs.clone(),
            log_responses: LogResponse::new_vec(
                &logs,
                13u32.into(),
                [12u8; 32].into(),
                [10u8; 32].into(),
                11u32.into(),
            ),
            gas_used: 5u64.into(),
            from: [6u8; 20].into(),
            to: Some([7u8; 20].into()),
            contract_address: Some([8u8; 20].into()),
            logs_bloom: [9u8; 256].into(),
            hash: [10u8; 32].into(),
            block_number: 11u64.into(),
            block_timestamp: 12u64.into(),
            transaction_hash: [12u8; 32].into(),
            transaction_index: 13u64.into(),
            cumulative_gas_used: 14u64.into(),
            nonce: 15u64.into(),
            result_bytes: None,
            effective_gas_price: 0u64.into(),
            transaction_type: 0,
        };
        let bytes = tx_receipt_ed.encode();
        let decoded = TxReceiptED::decode(bytes).unwrap();
        assert_eq!(tx_receipt_ed, decoded);
    }
}
