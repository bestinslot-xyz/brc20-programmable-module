use std::error::Error;

use revm::primitives::alloy_primitives::Bytes;
use revm_state::Bytecode;
use serde::Serialize;

use crate::db::types::{Decode, Encode};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BytecodeED(pub Bytecode);

impl Serialize for BytecodeED {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let hex_string = format!("{:x}", self.0.bytecode());
        serializer.serialize_str(&hex_string)
    }
}

impl Encode for BytecodeED {
    fn encode(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.0.bytecode().0);
        Ok(bytes)
    }
}

impl Decode for BytecodeED {
    fn decode(bytes: Vec<u8>) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        Ok(BytecodeED(Bytecode::new_raw(Bytes::from(bytes))))
    }
}

#[cfg(test)]
mod tests {
    use revm::primitives::Bytes;

    use super::*;

    #[test]
    fn test_bytecode_ed() {
        let bytecode: Bytecode = Bytecode::new_raw(Bytes::from("Hello world"));
        let bytecode_ed = BytecodeED(bytecode);
        let bytes = BytecodeED::encode(&bytecode_ed).unwrap();
        let decoded = BytecodeED::decode(bytes).unwrap();
        assert_eq!(bytecode_ed.0, decoded.0);
    }

    #[test]
    fn test_bytecode_ed_empty() {
        let bytecode: Bytecode = Bytecode::new_raw(Bytes::from(""));
        let bytecode_ed = BytecodeED(bytecode);
        let bytes = BytecodeED::encode(&bytecode_ed).unwrap();
        let decoded = BytecodeED::decode(bytes).unwrap();
        assert_eq!(bytecode_ed.0, decoded.0);
    }

    #[test]
    fn test_bytecode_ed_serialize() {
        let bytecode: Bytecode = Bytecode::new_raw(Bytes::from("Hello world"));
        let bytecode_ed = BytecodeED(bytecode);
        let serialized = serde_json::to_string(&bytecode_ed).unwrap();
        assert_eq!(serialized, "\"0x48656c6c6f20776f726c64\"");
    }
}
