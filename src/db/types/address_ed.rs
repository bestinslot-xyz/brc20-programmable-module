use std::error::Error;

use revm::primitives::Address;
use serde::Serialize;

use crate::db::types::{Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AddressED(pub Address);

impl Serialize for AddressED {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let hex_string = format!("0x{:x}", self.0);
        serializer.serialize_str(&hex_string)
    }
}

impl Encode for AddressED {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.0.to_vec());
        bytes
    }
}

impl Decode for AddressED {
    fn decode(bytes: Vec<u8>) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let mut bytes_array = [0u8; 20];
        bytes_array.copy_from_slice(&bytes);
        Ok(AddressED(bytes_array.into()))
    }
}

#[cfg(test)]
mod tests {
    use revm::primitives::Address;

    use super::*;

    #[test]
    fn test_address_ed() {
        let address: Address = "0x1234567890123456789012345678901234567890"
            .parse()
            .unwrap();
        let address_ed = AddressED(address);
        let bytes = address_ed.encode();
        let decoded = AddressED::decode(bytes).unwrap();
        assert_eq!(address_ed.0, decoded.0);
    }

    #[test]
    fn test_address_ed_empty() {
        let address: Address = Address::ZERO;
        let address_ed = AddressED(address);
        let bytes = address_ed.encode();
        let decoded = AddressED::decode(bytes).unwrap();
        assert_eq!(address_ed.0, decoded.0);
    }

    #[test]
    fn test_address_ed_serialize() {
        let address: Address = "0x1234567890123456789012345678901234567890"
            .parse()
            .unwrap();
        let address_ed = AddressED(address);
        let serialized = serde_json::to_string(&address_ed).unwrap();
        assert_eq!(serialized, "\"0x1234567890123456789012345678901234567890\"");
    }
}
