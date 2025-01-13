use serde::Serialize;

use revm::primitives::{ keccak256, Address, ExecutionResult, HaltReason, OutOfGasError, Output, SuccessReason, B256, U256 };
use revm::primitives::alloy_primitives::Bytes;

#[derive(Serialize)]
pub struct BlockResJSON{
  pub number: String,
  pub timestamp: String,
  #[serde(rename = "gasUsed")]
  pub gas_used: String,
  #[serde(rename = "mineTm")]
  pub mine_tm: String,
  pub hash: String,
}

#[derive(Serialize)]
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
  pub logs: Vec<SerializeableLog>,
  #[serde(rename = "callOutput")]
  pub call_output: Option<String>,
  #[serde(rename = "contractAddress")]
  pub contract_address: Option<String>,
}
#[derive(Serialize)]
pub struct SerializeableLog {
  pub address: String,
  pub topics: Vec<String>,
  pub data: String,
}

pub struct BlockRes {
  pub number: U256,
  pub timestamp: U256,
  pub gas_used: U256,
  pub mine_tm: U256,
  pub hash: B256,
}
pub struct TxInfo {
  pub from: Address,
  pub to: Option<Address>,
  pub data: Bytes,
}


pub fn get_serializeable_execution_result(old: ExecutionResult, txhash: String, nonce: u64) -> SerializableExecutionResult {
  match old {
    ExecutionResult::Success { gas_used, gas_refunded, logs, output, reason } => {
      SerializableExecutionResult {
        txhash: "0x".to_string() + txhash.as_str(),
        nonce,
        r#type: "Success".to_string(),
        reason: match reason {
          SuccessReason::Stop => "Stop".to_string(),
          SuccessReason::Return => "Return".to_string(),
          SuccessReason::SelfDestruct => "SelfDestruct".to_string(),
        },
        gas_used: gas_used.to_string(),
        gas_refunded: gas_refunded.to_string(),
        logs: logs.iter().map(|log| SerializeableLog {
          address: log.address.to_string(),
          topics: log.topics().iter().map(|topic| format!("{:?}", topic)).collect(),
          data: format!("{}", hex::encode(&log.data.data)),
        }).collect(),
        call_output: match output.clone() {
          Output::Call(data) => Some(format!("{}", hex::encode(data))),
          Output::Create(_, _) => None,
        },
        contract_address: match output.clone() {
          Output::Call(_) => None,
          Output::Create(_, addr) => addr.map(|x| x.to_string()),
        },
      }
    },
    ExecutionResult::Revert { gas_used, output } => {
      SerializableExecutionResult {
        txhash,
        nonce,
        r#type: "Revert".to_string(),
        reason: "".to_string(),
        gas_used: gas_used.to_string(),
        gas_refunded: "0".to_string(),
        logs: Vec::new(),
        call_output: Some(format!("{}", hex::encode(output))),
        contract_address: None,
      }
    },
    ExecutionResult::Halt { gas_used, reason } => {
      SerializableExecutionResult {
        txhash,
        nonce,
        r#type: "Halt".to_string(),
        reason: match reason {
          HaltReason::OutOfGas(OutOfGasError::Basic) => "OutOfGas::Basic".to_string(),
          HaltReason::OutOfGas(OutOfGasError::MemoryLimit) => "OutOfGas::MemoryLimit".to_string(),
          HaltReason::OutOfGas(OutOfGasError::Memory) => "OutOfGas::Memory".to_string(),
          HaltReason::OutOfGas(OutOfGasError::Precompile) => "OutOfGas::Precompile".to_string(),
          HaltReason::OutOfGas(OutOfGasError::InvalidOperand) => "OutOfGas::InvalidOperand".to_string(),
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
        },
        gas_used: gas_used.to_string(),
        gas_refunded: "0".to_string(),
        logs: Vec::new(),
        call_output: None,
        contract_address: None,
      }
    },
  }
}

pub fn get_tx_hash(txinfo: &TxInfo, nonce: &u64) -> String {
  let mut data = Vec::new();
  data.extend_from_slice(txinfo.from.as_slice());
  data.extend_from_slice(&nonce.to_be_bytes());
  if let Some(to) = txinfo.to {
    data.extend_from_slice(to.as_slice());
  } else {
    data.extend_from_slice(&[0; 20]);
  }
  data.extend_from_slice(&txinfo.data);
  hex::encode(keccak256(data).0)
}