#![cfg(feature = "server")]

use std::error::Error;

use alloy::consensus::{
    Block, BlockBody, Header, Receipt, ReceiptWithBloom, SignableTransaction, TxEnvelope, TxLegacy,
};
use alloy::primitives::{Bloom, Log, B64};
use alloy::signers::Signature;
use alloy_rlp::{Decodable, Encodable, RlpDecodable, RlpEncodable};
use revm::primitives::{Address, Bytes, LogData, TxKind, B256, U256};

use crate::db::types::{Decode, Encode};
use crate::types::{BlockResponseED, TxED, TxReceiptED};

#[derive(RlpEncodable, RlpDecodable, Debug, Clone, PartialEq, Eq)]
pub struct RawBlock {
    pub block: Block<TxEnvelope>,
    pub receipts: Vec<ReceiptWithBloom>,
}

impl RawBlock {
    pub fn new(
        block: BlockResponseED,
        transactions: Vec<TxED>,
        receipts: Vec<TxReceiptED>,
    ) -> Self {
        Self {
            block: Block::<TxEnvelope> {
                header: Header {
                    parent_hash: block.parent_hash.bytes,
                    ommers_hash: B256::ZERO,
                    beneficiary: Address::ZERO,
                    state_root: block.state_root.bytes,
                    transactions_root: block.transactions_root.bytes,
                    receipts_root: block.receipts_root.bytes,
                    logs_bloom: Bloom(block.logs_bloom.bytes),
                    difficulty: U256::from(block.difficulty.uint),
                    number: block.number.into(),
                    gas_limit: block.gas_limit.into(),
                    gas_used: block.gas_used.into(),
                    timestamp: block.timestamp.into(),
                    extra_data: Bytes::new(),
                    mix_hash: B256::ZERO,
                    nonce: B64::from_slice(&block.nonce.uint.to_be_bytes::<8>()),
                    base_fee_per_gas: Some(block.base_fee_per_gas.into()),
                    withdrawals_root: Some(B256::ZERO),
                    blob_gas_used: Some(block.blob_gas_used.into()),
                    excess_blob_gas: Some(block.excess_blob_gas.into()),
                    parent_beacon_block_root: Some(B256::ZERO),
                    requests_hash: Some(B256::ZERO),
                },
                body: BlockBody {
                    transactions: transactions
                        .into_iter()
                        .map(|tx| {
                            TxEnvelope::Legacy(
                                TxLegacy {
                                    nonce: tx.nonce.into(),
                                    to: match tx.to {
                                        Some(addr) => {
                                            if addr.address.is_zero() {
                                                TxKind::Create
                                            } else {
                                                TxKind::Call(addr.address.into())
                                            }
                                        }
                                        None => TxKind::Create,
                                    },
                                    value: U256::from(tx.value.uint),
                                    gas_price: 0,
                                    input: tx.input.bytes,
                                    chain_id: Some(tx.chain_id.into()),
                                    gas_limit: tx.gas.into(),
                                }
                                .into_signed(Signature::new(
                                    tx.r.uint,
                                    tx.s.uint,
                                    (!tx.v.is_zero()).into(),
                                )),
                            )
                        })
                        .collect(),
                    ommers: Vec::new(),
                    withdrawals: None,
                },
            },
            receipts: receipts
                .into_iter()
                .map(|r| ReceiptWithBloom::<Receipt> {
                    receipt: Receipt {
                        status: (!r.status.uint.is_zero()).into(),
                        cumulative_gas_used: r.cumulative_gas_used.into(),
                        logs: r
                            .logs
                            .into_iter()
                            .map(|log| Log {
                                address: log.address.address,
                                data: LogData::new_unchecked(
                                    log.topics.into_iter().map(|t| t.bytes).collect(),
                                    log.data.bytes,
                                ),
                            })
                            .collect(),
                    },
                    logs_bloom: Bloom(r.logs_bloom.bytes),
                })
                .collect(),
        }
    }

    pub fn raw_header(&self) -> String {
        let mut raw_bytes = Vec::new();
        self.block.header.encode(&mut raw_bytes);
        "0x".to_string() + hex::encode(raw_bytes).as_str()
    }

    pub fn raw_block(&self) -> String {
        let mut raw_bytes = Vec::new();
        self.block.encode(&mut raw_bytes);
        "0x".to_string() + hex::encode(raw_bytes).as_str()
    }

    pub fn raw_receipts(&self) -> Vec<String> {
        let mut raw_receipts = Vec::new();
        for receipt in &self.receipts {
            let mut raw_bytes = Vec::new();
            receipt.encode(&mut raw_bytes);
            raw_receipts.push("0x".to_string() + hex::encode(raw_bytes).as_str());
        }
        raw_receipts
    }
}

impl Encode for RawBlock {
    fn encode(&self, mut buffer: &mut Vec<u8>) {
        Encode::encode(&self.raw_block(), &mut buffer);
        Encode::encode(&self.raw_receipts(), &mut buffer);
    }
}

impl Decode for RawBlock {
    fn decode(bytes: &[u8], offset: usize) -> Result<(Self, usize), Box<dyn Error>> {
        let (block_string, offset): (String, usize) = Decode::decode(bytes, offset)?;
        let block_bytes = hex::decode(block_string.trim_start_matches("0x"))?;
        let block = Block::<TxEnvelope>::decode(&mut block_bytes.as_slice())?;

        let (receipts_strings, offset): (Vec<String>, usize) = Decode::decode(bytes, offset)?;
        let mut receipts = Vec::new();
        for receipt_string in receipts_strings {
            let receipt_bytes = hex::decode(receipt_string.trim_start_matches("0x"))?;
            let receipt = ReceiptWithBloom::decode(&mut receipt_bytes.as_slice())?;
            receipts.push(receipt);
        }

        Ok((RawBlock { block, receipts }, offset))
    }
}
