use std::error::Error;

use revm_state::Bytecode;
use serde::Serialize;

use crate::db::types::{Decode, Encode};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct BytecodeED {
    pub bytecode: Bytecode,
}

impl Serialize for BytecodeED {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let hex_string = format!("{:x}", self.bytecode.original_bytes());
        serializer.serialize_str(&hex_string)
    }
}

impl Encode for BytecodeED {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.bytecode.original_byte_slice());
        bytes
    }
}

impl Decode for BytecodeED {
    fn decode(bytes: Vec<u8>) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        Ok(Self {
            bytecode: Bytecode::new_raw(bytes.into()),
        })
    }
}

impl From<Bytecode> for BytecodeED {
    fn from(bytecode: Bytecode) -> Self {
        Self { bytecode }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytecode_ed() {
        let bytecode_ed: BytecodeED = Bytecode::new_raw("Hello world".into()).into();
        let bytes = bytecode_ed.encode();
        let decoded = BytecodeED::decode(bytes).unwrap();
        assert_eq!(bytecode_ed, decoded);
    }

    #[test]
    fn test_bytecode_ed_empty() {
        let bytecode_ed: BytecodeED = Bytecode::new_raw("".into()).into();
        let bytes = bytecode_ed.encode();
        let decoded = BytecodeED::decode(bytes).unwrap();
        assert_eq!(bytecode_ed, decoded);
    }

    #[test]
    fn test_bytecode_ed_serialize() {
        let bytecode_ed: BytecodeED = Bytecode::new_raw("Hello world ".into()).into();
        let serialized = serde_json::to_string(&bytecode_ed).unwrap();
        assert_eq!(serialized, "\"0x48656c6c6f20776f726c6420\"");
    }
}
