use alloy_primitives::{Bytes, U256};
use alloy_rpc_types_trace::geth::CallFrame;
use serde::Serialize;

use crate::db::types::{AddressED, BytesED, Decode, Encode, U256ED};

#[derive(Serialize, Clone, PartialEq, Eq, Debug)]
pub struct TraceED {
    #[serde(rename = "type")]
    pub tx_type: String,
    pub from: AddressED,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<AddressED>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub calls: Vec<TraceED>,
    pub gas: U256ED,
    #[serde(rename = "gasUsed")]
    pub gas_used: U256ED,
    pub input: BytesED,
    pub output: BytesED,
    pub value: U256ED,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl TraceED {
    pub fn new(call: &CallFrame) -> Self {
        Self {
            tx_type: call.typ.clone(),
            from: AddressED(call.from),
            to: call.to.map(AddressED),
            calls: call.calls.iter().map(TraceED::new).collect(),
            gas: U256ED::from_u256(call.gas),
            gas_used: U256ED::from_u256(call.gas_used),
            input: BytesED(call.input.clone()),
            output: BytesED(call.output.clone().unwrap_or(Bytes::new())),
            value: U256ED::from_u256(call.value.unwrap_or(U256::ZERO)),
            error: call.error.clone(),
        }
    }
}

impl Encode for TraceED {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let type_bytes = self.tx_type.as_bytes();
        bytes.extend_from_slice(&(type_bytes.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&type_bytes);
        bytes.extend_from_slice(&self.from.encode());
        if let Some(to) = &self.to {
            bytes.extend_from_slice(&to.encode());
        } else {
            bytes.extend_from_slice(&[0; 20]);
        }
        bytes.extend_from_slice(&(self.calls.len() as u64).to_be_bytes());
        for call in &self.calls {
            let call_bytes = call.encode();
            bytes.extend_from_slice(&(call_bytes.len() as u32).to_be_bytes());
            bytes.extend_from_slice(&call_bytes);
        }
        bytes.extend_from_slice(&self.gas.encode());
        bytes.extend_from_slice(&self.gas_used.encode());

        let input_bytes = &self.input.encode();
        bytes.extend_from_slice(&(input_bytes.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&self.input.encode());

        let output_bytes = &self.output.encode();
        bytes.extend_from_slice(&(output_bytes.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&self.output.encode());

        bytes.extend_from_slice(&self.value.encode());
        if let Some(error) = &self.error {
            let error_bytes = error.as_bytes();
            bytes.extend_from_slice(&(error_bytes.len() as u32).to_be_bytes());
            bytes.extend_from_slice(&error.as_bytes());
        }
        bytes
    }
}

impl Decode for TraceED {
    fn decode(bytes: Vec<u8>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut i = 0;
        let type_len = u32::from_be_bytes(bytes[i..i + 4].try_into()?);
        i += 4;
        let tx_type = String::from_utf8(bytes[i..i + type_len as usize].to_vec())?;
        i += type_len as usize;
        let from = AddressED::decode(bytes[i..i + 20].to_vec())?;
        i += 20;
        let to = AddressED::decode(bytes[i..i + 20].to_vec()).ok();
        i += 20;
        let calls_count = u64::from_be_bytes(bytes[i..i + 8].try_into()?);
        i += 8;
        let mut calls = Vec::new();
        for _ in 0..calls_count {
            let call_len = u32::from_be_bytes(bytes[i..i + 4].try_into()?);
            i += 4;
            calls.push(TraceED::decode(bytes[i..i + call_len as usize].to_vec())?);
            i += call_len as usize;
        }
        let gas = U256ED::decode(bytes[i..i + 32].to_vec())?;
        i += 32;
        let gas_used = U256ED::decode(bytes[i..i + 32].to_vec())?;
        i += 32;

        let input_len = u32::from_be_bytes(bytes[i..i + 4].try_into()?);
        i += 4;
        let input = BytesED::decode(bytes[i..i + input_len as usize].to_vec())?;
        i += input_len as usize;

        let output_len = u32::from_be_bytes(bytes[i..i + 4].try_into()?);
        i += 4;
        let output = BytesED::decode(bytes[i..i + output_len as usize].to_vec())?;
        i += output_len as usize;

        let value = U256ED::decode(bytes[i..i + 32].to_vec())?;
        i += 32;

        let error_len = u32::from_be_bytes(bytes[i..i + 4].try_into()?);
        i += 4;
        let error = if error_len > 0 {
            Some(String::from_utf8(
                bytes[i..i + error_len as usize].to_vec(),
            )?)
        } else {
            None
        };
        Ok(TraceED {
            tx_type,
            from,
            to,
            calls,
            gas,
            gas_used,
            input,
            output,
            value,
            error,
        })
    }
}
