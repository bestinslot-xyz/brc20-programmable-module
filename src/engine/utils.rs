use std::error::Error;
use std::time::{Duration, Instant};

use alloy::consensus::TxLegacy;
use alloy::primitives::{keccak256, Address, Bytes, B256, U256};
use revm::context::result::{ExecutionResult, HaltReason, OutOfGasError, Output, SuccessReason};
use revm::primitives::TxKind;

use crate::global::GAS_PER_BYTE;

/// This struct is used to store the unfinalised block information
pub struct LastBlockInfo {
    pub waiting_tx_count: u64,
    pub timestamp: u64,
    pub hash: B256,
    pub gas_used: u64,
    pub log_index: u64,
    pub start_time: Instant,
    pub total_processing_time: Option<Duration>,
}

impl LastBlockInfo {
    pub fn new() -> Self {
        LastBlockInfo {
            waiting_tx_count: 0,
            timestamp: 0,
            hash: B256::ZERO,
            gas_used: 0,
            log_index: 0,
            start_time: Instant::now(),
            total_processing_time: None,
        }
    }
}

#[derive(Clone)]
pub struct TxInfo {
    pub from: Address,
    pub to: TxKind,
    pub data: Bytes,
    pub nonce: Option<u64>,
    pub pre_hash: Option<B256>,
    pub v: u8,
    pub r: U256,
    pub s: U256,
}

impl TxInfo {
    pub fn from_inscription(from: Address, to: TxKind, data: Bytes) -> Self {
        TxInfo {
            from,
            to,
            data,
            nonce: None,
            pre_hash: None,
            v: 0,
            r: U256::ZERO,
            s: U256::ZERO,
        }
    }

    pub fn from_raw_transaction(
        from: Address,
        raw_tx: TxLegacy,
        tx_hash: B256,
        v: u8,
        r: U256,
        s: U256,
    ) -> Self {
        TxInfo {
            from,
            to: match raw_tx.to.into_to() {
                Some(to) => {
                    if to == Address::ZERO {
                        TxKind::Create
                    } else {
                        TxKind::Call(to)
                    }
                }
                None => TxKind::Create,
            },
            data: raw_tx.input,
            nonce: Some(raw_tx.nonce),
            pre_hash: Some(tx_hash),
            v,
            r,
            s,
        }
    }

    pub fn from_saved_transaction(
        from: Address,
        to: TxKind,
        data: Bytes,
        nonce: u64,
        tx_hash: B256,
        v: u8,
        r: U256,
        s: U256,
    ) -> Self {
        TxInfo {
            from,
            to,
            data,
            nonce: Some(nonce),
            pre_hash: Some(tx_hash),
            v,
            r,
            s,
        }
    }

    pub fn to_address_optional(&self) -> Option<Address> {
        match self.to {
            TxKind::Call(to) => Some(to),
            TxKind::Create => None,
        }
    }
}

pub fn get_tx_hash(tx_info: &TxInfo, account_nonce: u64) -> B256 {
    if let Some(pre_hash) = tx_info.pre_hash {
        return pre_hash;
    }
    let mut data = Vec::new();
    data.extend_from_slice(tx_info.from.as_slice());
    data.extend_from_slice(&tx_info.nonce.unwrap_or(account_nonce).to_be_bytes());
    if let TxKind::Call(to) = tx_info.to {
        data.extend_from_slice(to.as_slice());
    } else {
        data.extend_from_slice(&[0; 20]);
    }
    data.extend_from_slice(&tx_info.data);
    keccak256(data)
}

pub fn get_gas_limit(inscription_byte_len: u64) -> u64 {
    inscription_byte_len.saturating_mul(GAS_PER_BYTE)
}

pub fn get_inscription_byte_len(gas_limit: u64) -> u64 {
    gas_limit.saturating_div(GAS_PER_BYTE)
}

pub fn get_evm_address_from_pkscript(pkscript: &str) -> Result<Address, Box<dyn Error>> {
    let pkscript_bytes = hex::decode(pkscript)?;
    let mut address = [0u8; 20];
    let pkscript_hash = keccak256(pkscript_bytes);
    address.copy_from_slice(&pkscript_hash[12..32]);
    Ok(Address::from_slice(&address))
}

pub fn get_result_type(result: &ExecutionResult) -> String {
    match result {
        ExecutionResult::Success {
            gas_used: _,
            gas_refunded: _,
            logs: _,
            output: _,
            reason: _,
        } => "Success".to_string(),
        ExecutionResult::Revert {
            gas_used: _,
            output: _,
        } => "Revert".to_string(),
        ExecutionResult::Halt {
            gas_used: _,
            reason: _,
        } => "Halt".to_string(),
    }
}

pub fn get_result_reason(result: &ExecutionResult) -> String {
    match result {
        ExecutionResult::Success {
            gas_used: _,
            gas_refunded: _,
            logs: _,
            output: _,
            reason,
        } => match reason {
            SuccessReason::Stop => "Stop".to_string(),
            SuccessReason::Return => "Return".to_string(),
            SuccessReason::SelfDestruct => "SelfDestruct".to_string(),
            SuccessReason::EofReturnContract => "EofReturnContract".to_string(),
        },
        ExecutionResult::Revert {
            gas_used: _,
            output: _,
        } => "".to_string(),
        ExecutionResult::Halt {
            gas_used: _,
            reason,
        } => match reason {
            HaltReason::OutOfGas(OutOfGasError::Basic) => "OutOfGas::Basic".to_string(),
            HaltReason::OutOfGas(OutOfGasError::MemoryLimit) => "OutOfGas::MemoryLimit".to_string(),
            HaltReason::OutOfGas(OutOfGasError::Memory) => "OutOfGas::Memory".to_string(),
            HaltReason::OutOfGas(OutOfGasError::Precompile) => "OutOfGas::Precompile".to_string(),
            HaltReason::OutOfGas(OutOfGasError::InvalidOperand) => {
                "OutOfGas::InvalidOperand".to_string()
            }
            HaltReason::OutOfGas(OutOfGasError::ReentrancySentry) => {
                "OutOfGas::ReentrancySentry".to_string()
            }
            HaltReason::OpcodeNotFound => "OpcodeNotFound".to_string(),
            HaltReason::InvalidFEOpcode => "InvalidFEOpcode".to_string(),
            HaltReason::InvalidJump => "InvalidJump".to_string(),
            HaltReason::NotActivated => "NotActivated".to_string(),
            HaltReason::StackUnderflow => "StackUnderflow".to_string(),
            HaltReason::StackOverflow => "StackOverflow".to_string(),
            HaltReason::OutOfOffset => "OutOfOffset".to_string(),
            HaltReason::CreateCollision => "CreateCollision".to_string(),
            HaltReason::PrecompileError => "PrecompileError".to_string(),
            HaltReason::NonceOverflow => "NonceOverflow".to_string(),
            HaltReason::CreateContractSizeLimit => "CreateContractSizeLimit".to_string(),
            HaltReason::CreateContractStartingWithEF => "CreateContractStartingWithEF".to_string(),
            HaltReason::CreateInitCodeSizeLimit => "CreateInitCodeSizeLimit".to_string(),
            HaltReason::OverflowPayment => "OverflowPayment".to_string(),
            HaltReason::StateChangeDuringStaticCall => "StateChangeDuringStaticCall".to_string(),
            HaltReason::CallNotAllowedInsideStatic => "CallNotAllowedInsideStatic".to_string(),
            HaltReason::OutOfFunds => "OutOfFunds".to_string(),
            HaltReason::CallTooDeep => "CallTooDeep".to_string(),
            HaltReason::EofAuxDataOverflow => "EofAuxDataOverflow".to_string(),
            HaltReason::EofAuxDataTooSmall => "EofAuxDataTooSmall".to_string(),
            HaltReason::SubRoutineStackOverflow => "SubRoutineStackOverflow".to_string(),
            HaltReason::InvalidEXTCALLTarget => "InvalidEXTCALLTarget".to_string(),
        },
    }
}

pub fn get_contract_address(result: &ExecutionResult) -> Option<Address> {
    match result {
        ExecutionResult::Success {
            gas_used: _,
            gas_refunded: _,
            logs: _,
            output,
            reason: _,
        } => match output {
            Output::Call(_) => None,
            Output::Create(_, addr) => *addr,
        },
        ExecutionResult::Revert {
            gas_used: _,
            output: _,
        } => None,
        ExecutionResult::Halt {
            gas_used: _,
            reason: _,
        } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_evm_address() {
        let btc_pkscript = "76a914f1b8e7e4f3f1f2f1e1f1f1f1f1f1f1f1f1f1f1f188ac";
        let evm_address = get_evm_address_from_pkscript(btc_pkscript).unwrap();
        assert_eq!(
            evm_address.to_string().to_lowercase(),
            "0x7f217045127859b40ef1a27a5bfe73aa16687467"
        );
    }
}
