use std::time::Instant;

use revm::primitives::alloy_primitives::Bytes;
use revm::primitives::{keccak256, Address, B256};

/// This struct is used to store the unfinalised block information
pub struct LastBlockInfo {
    pub waiting_tx_count: u64,
    pub timestamp: u64,
    pub hash: B256,
    pub gas_used: u64,
    pub log_index: u64,
    pub start_time: Option<Instant>,
}

impl LastBlockInfo {
    pub fn new() -> Self {
        LastBlockInfo {
            waiting_tx_count: 0,
            timestamp: 0,
            hash: B256::ZERO,
            gas_used: 0,
            log_index: 0,
            start_time: None,
        }
    }
}

#[derive(Clone)]
pub struct TxInfo {
    pub from: Address,
    pub to: Option<Address>,
    pub data: Bytes,
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
