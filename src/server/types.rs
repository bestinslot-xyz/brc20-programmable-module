use revm::primitives::alloy_primitives::Bytes;
use revm::primitives::{keccak256, Address, B256};

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
