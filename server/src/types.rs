use serde::{Deserialize, Serialize};

use revm::primitives::alloy_primitives::{Bytes, U128};
use revm::primitives::{
    keccak256, Address, ExecutionResult, HaltReason, OutOfGasError, Output, SuccessReason, B256,
};

#[derive(Serialize, Clone, Debug)]
pub struct BlockResJSON {
    pub number: String,
    pub timestamp: String,
    #[serde(rename = "gasUsed")]
    pub gas_used: String,
    #[serde(rename = "mineTm")]
    pub mine_tm: String,
    pub hash: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct SerializableExecutionResult {
    pub txhash: String,
    pub nonce: u64,
    #[serde(rename = "txResult")]
    pub r#type: String,
    #[serde(rename = "endReason")]
    pub reason: String,
    #[serde(rename = "gasUsed")]
    pub gas_used: String,
    #[serde(rename = "gasRefunded")]
    pub gas_refunded: String,
    pub logs: Vec<SerializableLog>,
    #[serde(rename = "callOutput")]
    pub call_output: Option<String>,
    #[serde(rename = "contractAddress")]
    pub contract_address: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct SerializableLog {
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,
}

impl From<&revm::primitives::Log> for SerializableLog {
    fn from(log: &revm::primitives::Log) -> Self {
        SerializableLog {
            address: log.address.to_string(),
            topics: log
                .topics()
                .iter()
                .map(|topic| format!("{:?}", topic))
                .collect(),
            data: format!("{}", hex::encode(&log.data.data)),
        }
    }
}

pub struct BlockRes {
    pub number: u64,
    pub timestamp: u64,
    pub gas_used: u64,
    pub mine_tm: U128,
    pub hash: B256,
}

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

pub fn get_serializable_execution_result(
    result: &ExecutionResult,
    txhash: B256,
    nonce: u64,
) -> SerializableExecutionResult {
    match result {
        ExecutionResult::Success {
            gas_used,
            gas_refunded,
            logs,
            output,
            reason,
        } => SerializableExecutionResult {
            txhash: txhash.to_string(),
            nonce,
            r#type: "Success".to_string(),
            reason: match reason {
                SuccessReason::Stop => "Stop".to_string(),
                SuccessReason::Return => "Return".to_string(),
                SuccessReason::SelfDestruct => "SelfDestruct".to_string(),
                SuccessReason::EofReturnContract => "EofReturnContract".to_string(),
            },
            gas_used: gas_used.to_string(),
            gas_refunded: gas_refunded.to_string(),
            logs: logs
                .iter()
                .map(|log| SerializableLog {
                    address: log.address.to_string(),
                    topics: log
                        .topics()
                        .iter()
                        .map(|topic| format!("{:?}", topic))
                        .collect(),
                    data: format!("{}", hex::encode(&log.data.data)),
                })
                .collect(),
            call_output: match output.clone() {
                Output::Call(data) => Some(format!("{}", hex::encode(data))),
                Output::Create(_, _) => None,
            },
            contract_address: match output.clone() {
                Output::Call(_) => None,
                Output::Create(_, addr) => addr.map(|x| x.to_string()),
            },
        },
        ExecutionResult::Revert { gas_used, output } => SerializableExecutionResult {
            txhash: txhash.to_string(),
            nonce,
            r#type: "Revert".to_string(),
            reason: "".to_string(),
            gas_used: gas_used.to_string(),
            gas_refunded: "0".to_string(),
            logs: Vec::new(),
            call_output: Some(format!("{}", hex::encode(output))),
            contract_address: None,
        },
        ExecutionResult::Halt { gas_used, reason } => SerializableExecutionResult {
            txhash: txhash.to_string(),
            nonce,
            r#type: "Halt".to_string(),
            reason: match reason {
                HaltReason::OutOfGas(OutOfGasError::Basic) => "OutOfGas::Basic".to_string(),
                HaltReason::OutOfGas(OutOfGasError::MemoryLimit) => {
                    "OutOfGas::MemoryLimit".to_string()
                }
                HaltReason::OutOfGas(OutOfGasError::Memory) => "OutOfGas::Memory".to_string(),
                HaltReason::OutOfGas(OutOfGasError::Precompile) => {
                    "OutOfGas::Precompile".to_string()
                }
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
                HaltReason::CreateContractStartingWithEF => {
                    "CreateContractStartingWithEF".to_string()
                }
                HaltReason::CreateInitCodeSizeLimit => "CreateInitCodeSizeLimit".to_string(),
                HaltReason::OverflowPayment => "OverflowPayment".to_string(),
                HaltReason::StateChangeDuringStaticCall => {
                    "StateChangeDuringStaticCall".to_string()
                }
                HaltReason::CallNotAllowedInsideStatic => "CallNotAllowedInsideStatic".to_string(),
                HaltReason::OutOfFunds => "OutOfFunds".to_string(),
                HaltReason::CallTooDeep => "CallTooDeep".to_string(),
                HaltReason::EofAuxDataOverflow => "EofAuxDataOverflow".to_string(),
                HaltReason::EofAuxDataTooSmall => "EofAuxDataTooSmall".to_string(),
                HaltReason::EOFFunctionStackOverflow => "EOFFunctionStackOverflow".to_string(),
                HaltReason::InvalidEXTCALLTarget => "InvalidEXTCALLTarget".to_string(),
            },
            gas_used: gas_used.to_string(),
            gas_refunded: "0".to_string(),
            logs: Vec::new(),
            call_output: None,
            contract_address: None,
        },
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
