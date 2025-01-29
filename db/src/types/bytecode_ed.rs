use std::error::Error;

use revm::primitives::alloy_primitives::Bytes;
use revm::primitives::Bytecode;

use super::{Decode, Encode};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BytecodeED(pub Bytecode);

impl BytecodeED {
    pub fn from_bytecode(a: Bytecode) -> Self {
        Self(a)
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
    use revm::primitives::{Bytecode, Bytes};

    use crate::types::{BytecodeED, Decode, Encode};

    #[test]
    fn test_bytecode_ed() {
        let bytecode: Bytecode = Bytecode::new_raw(Bytes::from("Hello world"));
        let bytecode_ed = BytecodeED::from_bytecode(bytecode);
        let bytes = BytecodeED::encode(&bytecode_ed).unwrap();
        let decoded = BytecodeED::decode(bytes).unwrap();
        assert_eq!(bytecode_ed.0, decoded.0);
    }
}
