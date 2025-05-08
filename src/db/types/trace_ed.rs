use std::error::Error;

use alloy_primitives::{Bytes, U256};
use alloy_rpc_types_trace::geth::CallFrame;
use serde::{Deserialize, Serialize};

use crate::db::types::{AddressED, BytesED, Decode, Encode, U256ED};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
/// Represents a `callTracer` entry for `debug_traceTransaction` method in BRC2.0
///
/// Refer to [Geth callTracer API](https://geth.ethereum.org/docs/developers/evm-tracing/built-in-tracers#call-tracer) for more details.
pub struct TraceED {
    #[serde(rename = "type")]
    /// The type of the trace entry (e.g., "call", "create", etc.)
    pub tx_type: String,
    /// The address of the sender
    pub from: AddressED,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The address of the recipient (if applicable)
    pub to: Option<AddressED>,
    #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::new")]
    /// A list of nested trace entries (if applicable)
    pub calls: Vec<TraceED>,
    /// The gas limit for the transaction
    pub gas: U256ED,
    #[serde(rename = "gasUsed")]
    /// The amount of gas used by the transaction
    pub gas_used: U256ED,
    /// The input data for the transaction
    pub input: BytesED,
    /// The output data from the transaction
    pub output: BytesED,
    /// The value transferred in the transaction, 0 for BRC2.0
    pub value: U256ED,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// An error message if the transaction failed
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "revertReason")]
    /// Revert reason if the transaction failed
    pub revert_reason: Option<String>,
}

impl TraceED {
    pub(crate) fn new(call: CallFrame) -> Self {
        Self {
            tx_type: call.typ,
            from: call.from.into(),
            to: call.to.map(AddressED::new),
            calls: call.calls.iter().map(|x| x.clone().into()).collect(),
            gas: call.gas.into(),
            gas_used: call.gas_used.into(),
            input: call.input.into(),
            output: call.output.unwrap_or(Bytes::new()).into(),
            value: call.value.unwrap_or(U256::ZERO).into(),
            error: call.error,
            revert_reason: call.revert_reason,
        }
    }

    pub(crate) fn get_created_contract(&self) -> Option<AddressED> {
        if self.tx_type.to_lowercase() == "create" {
            if let Some(to) = &self.to {
                return Some(to.clone());
            }
        }
        return None;
    }
}

impl From<CallFrame> for TraceED {
    fn from(call: CallFrame) -> Self {
        TraceED::new(call)
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
        self.revert_reason.encode(buffer);
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
        let (revert_reason, offset) = Decode::decode(bytes, offset)?;

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
                revert_reason,
            },
            offset,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::types::Decode;

    #[test]
    fn test_trace_ed() {
        let trace = TraceED {
            tx_type: "call".to_string(),
            from: [0; 20].into(),
            to: Some([1; 20].into()),
            calls: vec![TraceED {
                tx_type: "call".to_string(),
                from: [2; 20].into(),
                to: Some([3; 20].into()),
                calls: vec![],
                gas: U256::from(21000).into(),
                gas_used: U256::from(21000).into(),
                input: vec![0x60, 0x00].into(),
                output: vec![0x00].into(),
                value: U256::from(0).into(),
                error: None,
                revert_reason: None,
            }],
            gas: U256::from(21000).into(),
            gas_used: U256::from(21001).into(),
            input: vec![0x60, 0x00].into(),
            output: vec![0x00].into(),
            value: U256::from(0).into(),
            error: None,
            revert_reason: None,
        };

        let mut buffer = Vec::new();
        trace.encode(&mut buffer);

        let (decoded_trace, _) = TraceED::decode(&buffer, 0).unwrap();

        assert_eq!(trace, decoded_trace);
    }

    #[test]
    fn test_trace_ed_serde() {
        let trace = TraceED {
            tx_type: "call".to_string(),
            from: [0; 20].into(),
            to: Some([1; 20].into()),
            calls: vec![TraceED {
                tx_type: "call".to_string(),
                from: [2; 20].into(),
                to: Some([3; 20].into()),
                calls: vec![],
                gas: U256::from(21000).into(),
                gas_used: U256::from(21000).into(),
                input: vec![0x60, 0x00].into(),
                output: vec![0x00].into(),
                value: U256::from(0).into(),
                error: None,
                revert_reason: None,
            }],
            gas: U256::from(21000).into(),
            gas_used: U256::from(21001).into(),
            input: vec![0x60, 0x00].into(),
            output: vec![0x00].into(),
            value: U256::from(0).into(),
            error: None,
            revert_reason: None,
        };

        let serialized = serde_json::to_string(&trace).unwrap();
        let deserialized: TraceED = serde_json::from_str(&serialized).unwrap();

        assert_eq!(trace, deserialized);
    }

    #[test]
    fn test_get_created_contract() {
        let trace = TraceED {
            tx_type: "create".to_string(),
            from: [0; 20].into(),
            to: Some([1; 20].into()),
            calls: vec![
                TraceED {
                    tx_type: "create".to_string(),
                    from: [2; 20].into(),
                    to: Some([3; 20].into()),
                    calls: vec![],
                    gas: U256::from(21000).into(),
                    gas_used: U256::from(21000).into(),
                    input: vec![0x60, 0x00].into(),
                    output: vec![0x00].into(),
                    value: U256::from(0).into(),
                    error: None,
                    revert_reason: None,
                },
                TraceED {
                    tx_type: "call".to_string(),
                    from: [4; 20].into(),
                    to: Some([5; 20].into()),
                    calls: vec![TraceED {
                        tx_type: "create".to_string(),
                        from: [6; 20].into(),
                        to: Some([7; 20].into()),
                        calls: vec![],
                        gas: U256::from(21000).into(),
                        gas_used: U256::from(21000).into(),
                        input: vec![0x60, 0x00].into(),
                        output: vec![0x00].into(),
                        value: U256::from(0).into(),
                        error: None,
                        revert_reason: None,
                    }],
                    gas: U256::from(21000).into(),
                    gas_used: U256::from(21000).into(),
                    input: vec![0x60, 0x00].into(),
                    output: vec![0x00].into(),
                    value: U256::from(0).into(),
                    error: None,
                    revert_reason: None,
                },
            ],
            gas: U256::from(21000).into(),
            gas_used: U256::from(21000).into(),
            input: vec![0x60, 0x00].into(),
            output: vec![0x00].into(),
            value: U256::from(0).into(),
            error: None,
            revert_reason: None,
        };

        assert_eq!(trace.get_created_contract(), Some([1; 20].into()));
    }
}
