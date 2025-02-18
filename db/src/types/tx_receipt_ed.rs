use revm::primitives::{alloy_primitives::logs_bloom, Address, ExecutionResult, B256};
use serde::Serialize;
use serde_hex::{CompactPfx, SerHex};

use super::{AddressED, Decode, Encode, LogED, B2048ED, B256ED};

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct TxReceiptED {
    #[serde(serialize_with = "one_or_zero")]
    pub status: u8,
    pub logs: LogED,
    #[serde(rename = "gasUsed", with = "SerHex::<CompactPfx>")]
    pub gas_used: u64,
    pub from: AddressED,
    pub to: Option<AddressED>,
    #[serde(rename = "contractAddress")]
    pub contract_address: Option<AddressED>,
    #[serde(rename = "logsBloom")]
    pub logs_bloom: B2048ED,
    pub hash: B256ED,
    #[serde(rename = "blockNumber", with = "SerHex::<CompactPfx>")]
    pub block_number: u64,
    #[serde(rename = "transactionHash")]
    pub transaction_hash: B256ED,
    #[serde(rename = "transactionIndex", with = "SerHex::<CompactPfx>")]
    pub transaction_index: u64,
    #[serde(rename = "cumulativeGasUsed", with = "SerHex::<CompactPfx>")]
    pub cumulative_gas_used: u64,
    #[serde(with = "SerHex::<CompactPfx>")]
    pub nonce: u64,
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
        block_number: u64,
        contract_address: Option<Address>,
        from: Address,
        to: Option<Address>,
        tx_hash: B256,
        tx_idx: u64,
        output: &ExecutionResult,
        cumulative_gas_used: u64,
        nonce: u64,
        start_log_index: u64,
    ) -> Self {
        let logs = LogED {
            logs: output.logs().to_vec(),
            log_index: start_log_index,
        };
        let logs_bloom = B2048ED::decode(logs_bloom(output.logs()).to_vec()).unwrap();
        TxReceiptED {
            status: output.is_success() as u8,
            logs,
            gas_used: output.gas_used(),
            from: AddressED(from),
            to: to.map(AddressED),
            contract_address: contract_address.map(AddressED),
            logs_bloom,
            hash: B256ED::from_b256(block_hash),
            block_number: block_number,
            transaction_hash: B256ED::from_b256(tx_hash),
            transaction_index: tx_idx,
            cumulative_gas_used,
            nonce,
        }
    }
}

impl Encode for TxReceiptED {
    fn encode(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut bytes = Vec::new();
        bytes.push(self.status);

        let logs_bytes = self.logs.encode()?;

        bytes.extend_from_slice(&(logs_bytes.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&logs_bytes);

        bytes.extend_from_slice(&self.gas_used.to_be_bytes());
        bytes.extend_from_slice(&self.from.encode()?);
        bytes.extend_from_slice(
            &self
                .to
                .as_ref()
                .unwrap_or(&AddressED(Address::ZERO))
                .encode()?,
        );

        bytes.extend_from_slice(
            &self
                .contract_address
                .as_ref()
                .unwrap_or(&AddressED(Address::ZERO))
                .encode()?,
        );

        bytes.extend_from_slice(&self.logs_bloom.encode()?);
        bytes.extend_from_slice(&self.hash.encode()?);
        bytes.extend_from_slice(&self.block_number.to_be_bytes());
        bytes.extend_from_slice(&self.transaction_hash.encode()?);
        bytes.extend_from_slice(&self.transaction_index.to_be_bytes());
        bytes.extend_from_slice(&self.cumulative_gas_used.to_be_bytes());
        bytes.extend_from_slice(&self.nonce.to_be_bytes());
        Ok(bytes)
    }
}

impl Decode for TxReceiptED {
    fn decode(bytes: Vec<u8>) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        let status = bytes[0];
        let mut i = 1;

        let logs_len = u32::from_be_bytes(bytes[i..i + 4].try_into().unwrap()) as usize;
        i += 4;

        let logs = LogED::decode(bytes[i..i + logs_len].to_vec())?;
        i += logs_len;

        let gas_used = u64::from_be_bytes(bytes[i..i + 8].try_into().unwrap());
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
        let block_number = u64::from_be_bytes(bytes[i..i + 8].try_into().unwrap());
        i += 8;
        let transaction_hash = B256ED::decode(bytes[i..i + 32].to_vec())?;
        i += 32;
        let transaction_index = u64::from_be_bytes(bytes[i..i + 8].try_into().unwrap());
        i += 8;
        let cumulative_gas_used = u64::from_be_bytes(bytes[i..i + 8].try_into().unwrap());
        i += 8;
        let nonce = u64::from_be_bytes(bytes[i..i + 8].try_into().unwrap());
        Ok(TxReceiptED {
            status,
            logs,
            gas_used,
            from,
            to: if to.0 == Address::ZERO {
                None
            } else {
                Some(to)
            },
            contract_address: if contract_address.0 == Address::ZERO {
                None
            } else {
                Some(contract_address)
            },
            logs_bloom,
            hash: block_hash,
            block_number,
            transaction_hash,
            transaction_index,
            cumulative_gas_used,
            nonce,
        })
    }
}

#[cfg(test)]
mod tests {
    use revm::primitives::{alloy_primitives::aliases::B2048, Address, Log, B256};

    use crate::types::{AddressED, BEncodeDecode, Decode, Encode, LogED, TxReceiptED};

    #[test]
    fn test_tx_receipt_ed() {
        let logs = LogED {
            logs: vec![Log::new(
                Address::from([1u8; 20]),
                vec![B256::from([2u8; 32])],
                vec![3u8; 32].into(),
            )
            .unwrap()],
            log_index: 0,
        };
        let tx_receipt_ed = TxReceiptED {
            status: 4,
            logs,
            gas_used: 5,
            from: AddressED(Address::from([6u8; 20])),
            to: Some(AddressED(Address::from([7u8; 20]))),
            contract_address: Some(AddressED(Address::from([8u8; 20]))),
            logs_bloom: BEncodeDecode(B2048::from([9u8; 256])),
            hash: BEncodeDecode(B256::from([10u8; 32])),
            block_number: 11,
            transaction_hash: BEncodeDecode(B256::from([12u8; 32])),
            transaction_index: 13,
            cumulative_gas_used: 14,
            nonce: 15,
        };
        let bytes = TxReceiptED::encode(&tx_receipt_ed).unwrap();
        let decoded = TxReceiptED::decode(bytes).unwrap();
        assert_eq!(tx_receipt_ed, decoded);
    }
}
