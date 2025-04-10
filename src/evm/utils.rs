use std::cmp::max;

use alloy_primitives::{keccak256, Address, Bytes};
use revm::context::result::{ExecutionResult, HaltReason, OutOfGasError, Output, SuccessReason};

static GAS_PER_BYTE: u64 = 12000; // 12M gas per byte
static MINIMUM_GAS_LIMIT: u64 = 384000; // GAS_PER_BYTE * 32

pub fn get_gas_limit(inscription_byte_len: u64) -> u64 {
    max(
        inscription_byte_len.saturating_mul(GAS_PER_BYTE),
        MINIMUM_GAS_LIMIT,
    )
}

pub fn get_evm_address(pkscript_bytes: &Bytes) -> Address {
    let mut address = [0u8; 20];
    let pkscript_hash = keccak256(pkscript_bytes);
    address.copy_from_slice(&pkscript_hash[12..32]);
    Address::from_slice(&address)
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
        let evm_address = get_evm_address(&hex::decode(btc_pkscript).unwrap().into());
        assert_eq!(
            evm_address,
            "0x7f217045127859b40ef1a27a5bfe73aa16687467"
                .parse::<Address>()
                .unwrap(),
        );
    }
}
