use std::{borrow::Cow, error::Error};

use heed::{BytesDecode, BytesEncode};
use revm::primitives::alloy_primitives::Bytes;
use revm::primitives::Bytecode;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BytecodeED(pub Bytecode);

impl BytecodeED {
    pub fn from_bytecode(a: Bytecode) -> Self {
        Self(a)
    }
}

impl<'a> BytesEncode<'a> for BytecodeED {
    type EItem = BytecodeED;

    fn bytes_encode(item: &'a Self::EItem) -> Result<Cow<'a, [u8]>, Box<dyn Error>> {
        let bytes = item.0.bytecode.0.to_vec();
        Ok(Cow::Owned(bytes))
    }
}
impl<'a> BytesDecode<'a> for BytecodeED {
    type DItem = BytecodeED;

    fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem, Box<dyn Error>> {
        Ok(BytecodeED(Bytecode::new_raw(Bytes::from(bytes.to_vec()))))
    }
}

#[cfg(test)]
mod tests {
    use heed::{BytesDecode, BytesEncode};
    use revm::primitives::{Bytecode, Bytes};

    use crate::types::BytecodeED;

    #[test]
    fn test_bytecode_ed() {
        let bytecode: Bytecode = Bytecode::new_raw(Bytes::from("Hello world"));
        let bytecode_ed = BytecodeED::from_bytecode(bytecode);
        let bytes = BytecodeED::bytes_encode(&bytecode_ed).unwrap();
        let decoded = BytecodeED::bytes_decode(&bytes).unwrap();
        assert_eq!(bytecode_ed.0, decoded.0);
    }
}
