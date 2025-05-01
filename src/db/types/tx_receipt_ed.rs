use std::error::Error;

use alloy_primitives::{logs_bloom, Address, Bytes, B256};
use revm::context::result::ExecutionResult;
use serde::{Deserialize, Serialize};

use super::U8ED;
use crate::db::types::{AddressED, BytesED, Decode, Encode, LogED, B2048ED, B256ED, U64ED};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TxReceiptED {
    pub status: U8ED,
    #[serde(rename = "txResult")]
    pub transaction_result: String,
    #[serde(rename = "reason")]
    pub reason: String,
    pub logs: Vec<LogED>,
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
    #[serde(rename = "type")]
    pub transaction_type: U8ED,
    pub nonce: U64ED,
    #[serde(rename = "output")]
    pub result_bytes: Option<BytesED>,
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
        Ok(TxReceiptED {
            status: (output.is_success() as u8).into(),
            transaction_result: r#type,
            reason,
            logs: LogED::new_vec(
                &output.logs().to_vec(),
                start_log_index.into(),
                tx_idx.clone(),
                tx_hash.into(),
                block_hash.into(),
                block_number.clone(),
            ),
            gas_used: output.gas_used().into(),
            from: from.into(),
            to: to.map(Into::<AddressED>::into),
            contract_address: contract_address.map(Into::<AddressED>::into),
            logs_bloom: B2048ED::decode_vec(&logs_bloom(output.logs()).to_vec())?,
            hash: block_hash.into(),
            block_number: block_number.clone(),
            block_timestamp: block_timestamp,
            transaction_hash: tx_hash.into(),
            transaction_index: tx_idx.clone(),
            cumulative_gas_used,
            nonce,
            result_bytes: output_bytes.map(|bytes| bytes.clone().into()),
            effective_gas_price: 0u64.into(),
            transaction_type: 0u8.into(),
        })
    }
}

impl Encode for TxReceiptED {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.status.encode(buffer);
        self.transaction_result.encode(buffer);
        self.reason.encode(buffer);
        self.logs.encode(buffer);
        self.gas_used.encode(buffer);
        self.from.encode(buffer);
        self.to.encode(buffer);
        self.contract_address.encode(buffer);
        self.logs_bloom.encode(buffer);
        self.hash.encode(buffer);
        self.block_number.encode(buffer);
        self.block_timestamp.encode(buffer);
        self.transaction_hash.encode(buffer);
        self.transaction_index.encode(buffer);
        self.cumulative_gas_used.encode(buffer);
        self.nonce.encode(buffer);
        self.result_bytes.encode(buffer);
    }
}

impl Decode for TxReceiptED {
    fn decode(bytes: &[u8], offset: usize) -> Result<(Self, usize), Box<dyn Error>> {
        let (status, offset) = Decode::decode(bytes, offset)?;
        let (r#type, offset) = Decode::decode(bytes, offset)?;
        let (reason, offset) = Decode::decode(bytes, offset)?;
        let (logs, offset) = Decode::decode(bytes, offset)?;
        let (gas_used, offset) = Decode::decode(bytes, offset)?;
        let (from, offset) = Decode::decode(bytes, offset)?;
        let (to, offset) = Decode::decode(bytes, offset)?;
        let (contract_address, offset) = Decode::decode(bytes, offset)?;
        let (logs_bloom, offset) = Decode::decode(bytes, offset)?;
        let (block_hash, offset) = Decode::decode(bytes, offset)?;
        let (block_number, offset) = Decode::decode(bytes, offset)?;
        let (block_timestamp, offset) = Decode::decode(bytes, offset)?;
        let (transaction_hash, offset) = Decode::decode(bytes, offset)?;
        let (transaction_index, offset) = Decode::decode(bytes, offset)?;
        let (cumulative_gas_used, offset) = Decode::decode(bytes, offset)?;
        let (nonce, offset) = Decode::decode(bytes, offset)?;
        let (result_bytes, offset) = Decode::decode(bytes, offset)?;
        Ok((
            TxReceiptED {
                status,
                transaction_result: r#type,
                reason,
                logs,
                gas_used,
                from,
                to,
                contract_address,
                logs_bloom,
                hash: block_hash,
                block_number,
                block_timestamp,
                transaction_hash,
                transaction_index,
                cumulative_gas_used,
                effective_gas_price: 0u64.into(),
                transaction_type: 0u8.into(),
                nonce,
                result_bytes,
            },
            offset,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx_receipt_ed() {
        let logs = LogED {
            address: [1u8; 20].into(),
            topics: vec![[2u8; 32].into(), [3u8; 32].into()],
            data: BytesED::from([4u8; 32].to_vec()),
            transaction_index: 5u64.into(),
            transaction_hash: [6u8; 32].into(),
            block_hash: [7u8; 32].into(),
            block_number: 8u64.into(),
            log_index: 9u64.into(),
        };
        let tx_receipt_ed = TxReceiptED {
            status: 4u8.into(),
            transaction_result: "type".to_string(),
            reason: "reason".to_string(),
            logs: vec![logs],
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
            transaction_type: 0u8.into(),
        };
        let bytes = tx_receipt_ed.encode_vec();
        let decoded = TxReceiptED::decode_vec(&bytes).unwrap();
        assert_eq!(tx_receipt_ed, decoded);
    }

    #[test]
    fn test_tx_receipt_ed_serde() {
        let logs = LogED {
            address: [1u8; 20].into(),
            topics: vec![[2u8; 32].into(), [3u8; 32].into()],
            data: BytesED::from([4u8; 32].to_vec()),
            transaction_index: 5u64.into(),
            transaction_hash: [6u8; 32].into(),
            block_hash: [7u8; 32].into(),
            block_number: 8u64.into(),
            log_index: 9u64.into(),
        };
        let tx_receipt_ed = TxReceiptED {
            status: 4u8.into(),
            transaction_result: "type".to_string(),
            reason: "reason".to_string(),
            logs: vec![logs],
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
            transaction_type: 0u8.into(),
        };
        let serialized = serde_json::to_string(&tx_receipt_ed).unwrap();
        let deserialized: TxReceiptED = serde_json::from_str(&serialized).unwrap();
        assert_eq!(tx_receipt_ed, deserialized);
    }
}
