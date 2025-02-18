use serde::Deserialize;

use revm::primitives::alloy_primitives::Bytes;
use revm::primitives::{
    keccak256, Address, ExecutionResult, HaltReason, OutOfGasError, Output, SuccessReason, B256,
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
            HaltReason::EOFFunctionStackOverflow => "EOFFunctionStackOverflow".to_string(),
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
