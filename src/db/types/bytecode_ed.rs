use std::error::Error;

use revm_bytecode::Bytecode;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::db::types::{Decode, Encode};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
/// Represents a deployed bytecode in the EVM.
pub struct BytecodeED {
    /// The bytecode of the contract.
    pub bytecode: Bytecode,
}

impl BytecodeED {
    // This is returned by the API, so doesn't need to be public
    pub(crate) fn new(bytecode: Bytecode) -> Self {
        Self { bytecode }
    }
}

impl<'de> Deserialize<'de> for BytecodeED {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_string = String::deserialize(deserializer)?;
        Ok(BytecodeED {
            // Return an error if the bytecode is invalid, as this will run in the client
            bytecode: Bytecode::new_raw_checked(
                hex_string.parse().map_err(serde::de::Error::custom)?,
            )
            .map_err(serde::de::Error::custom)?,
        })
    }
}

impl Serialize for BytecodeED {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_string = format!("{:x}", self.bytecode.original_bytes());
        serializer.serialize_str(&hex_string)
    }
}

impl Encode for BytecodeED {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.bytecode.original_byte_slice().to_vec().encode(buffer);
    }
}

impl Decode for BytecodeED {
    fn decode(bytes: &[u8], offset: usize) -> Result<(Self, usize), Box<dyn Error>>
    where
        Self: Sized,
    {
        Vec::<u8>::decode(bytes, offset).map(|(bytes, offset)| {
            (
                BytecodeED {
                    // Panic if the bytecode is invalid as it's coming from the database
                    bytecode: Bytecode::new_raw_checked(bytes.into()).expect("Valid bytecode"),
                },
                offset,
            )
        })
    }
}

impl From<Bytecode> for BytecodeED {
    fn from(bytecode: Bytecode) -> Self {
        BytecodeED::new(bytecode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytecode_ed() {
        let bytecode_ed: BytecodeED = Bytecode::new_raw("Hello world".into()).into();
        let bytes = bytecode_ed.encode_vec();
        let decoded = BytecodeED::decode_vec(&bytes).unwrap();
        assert_eq!(bytecode_ed, decoded);
    }

    #[test]
    fn test_bytecode_ed_empty() {
        let bytecode_ed: BytecodeED = Bytecode::new_raw("".into()).into();
        let bytes = bytecode_ed.encode_vec();
        let decoded = BytecodeED::decode_vec(&bytes).unwrap();
        assert_eq!(bytecode_ed, decoded);
    }

    #[test]
    fn test_bytecode_ed_serialize() {
        let bytecode_ed: BytecodeED = Bytecode::new_raw("Hello world ".into()).into();
        let serialized = serde_json::to_string(&bytecode_ed).unwrap();
        let deserialized: BytecodeED = serde_json::from_str(&serialized).unwrap();
        assert_eq!(bytecode_ed, deserialized);
    }
}
