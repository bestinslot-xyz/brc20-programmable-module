use std::{borrow::Cow, error::Error};

use heed::{BytesDecode, BytesEncode};
use revm::primitives::alloy_primitives::Bytes;
use revm::primitives::Bytecode;

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
