use std::error::Error;

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

impl From<&CallFrame> for TraceED {
    fn from(call: &CallFrame) -> Self {
        Self {
            tx_type: call.typ.clone(),
            from: call.from.into(),
            to: call.to.map(Into::<AddressED>::into),
            calls: call.calls.iter().map(Into::<TraceED>::into).collect(),
            gas: call.gas.into(),
            gas_used: call.gas_used.into(),
            input: call.input.clone().into(),
            output: call.output.clone().unwrap_or(Bytes::new()).into(),
            value: call.value.unwrap_or(U256::ZERO).into(),
            error: call.error.clone(),
        }
    }
}

impl Encode for TraceED {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.tx_type.encode(buffer);
        self.from.encode(buffer);
        self.to.encode(buffer);
        self.calls.encode(buffer);
        self.gas.encode(buffer);
        self.gas_used.encode(buffer);
        self.input.encode(buffer);
        self.output.encode(buffer);
        self.value.encode(buffer);
        self.error.encode(buffer);
    }
}

impl Decode for TraceED {
    fn decode(bytes: &[u8], offset: usize) -> Result<(Self, usize), Box<dyn Error>> {
        let (tx_type, offset) = Decode::decode(bytes, offset)?;
        let (from, offset) = Decode::decode(bytes, offset)?;
        let (to, offset) = Decode::decode(bytes, offset)?;
        let (calls, offset) = Decode::decode(bytes, offset)?;
        let (gas, offset) = Decode::decode(bytes, offset)?;
        let (gas_used, offset) = Decode::decode(bytes, offset)?;
        let (input, offset) = Decode::decode(bytes, offset)?;
        let (output, offset) = Decode::decode(bytes, offset)?;
        let (value, offset) = Decode::decode(bytes, offset)?;
        let (error, offset) = Decode::decode(bytes, offset)?;

        Ok((
            TraceED {
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
            },
            offset,
        ))
    }
}
