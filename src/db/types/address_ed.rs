use std::error::Error;

use alloy_primitives::Address;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    db::types::{Decode, Encode},
    server::api::INVALID_ADDRESS,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AddressED {
    pub address: Address,
}

impl From<[u8; 20]> for AddressED {
    fn from(address: [u8; 20]) -> Self {
        Self {
            address: Address::new(address),
        }
    }
}

impl From<Address> for AddressED {
    fn from(address: Address) -> Self {
        Self { address }
    }
}

impl<'de> Deserialize<'de> for AddressED {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_string: String = String::deserialize(deserializer)?;
        Ok(AddressED {
            address: hex_string.parse().unwrap_or(*INVALID_ADDRESS),
        })
    }
}

impl Serialize for AddressED {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_string = format!("0x{:x}", self.address);
        serializer.serialize_str(&hex_string)
    }
}

impl Encode for AddressED {
    fn encode(&self, buffer: &mut Vec<u8>) {
        // Encode as FixedBytes<20>, i.e. [u8; 20]
        self.address.0.encode(buffer);
    }
}

impl Decode for AddressED {
    fn decode(bytes: &[u8], offset: usize) -> Result<(Self, usize), Box<dyn Error>>
    where
        Self: Sized,
    {
        <[u8; 20]>::decode(bytes, offset)
            .map(|(bytes, offset)| (Address::from_slice(&bytes).into(), offset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_ed() {
        let address: Address = "0x1234567890123456789012345678901234567890"
            .parse()
            .unwrap();
        let address_ed: AddressED = address.into();
        let bytes = address_ed.encode_vec();
        let decoded = AddressED::decode_vec(&bytes).unwrap();
        assert_eq!(address_ed, decoded);
    }

    #[test]
    fn test_address_ed_empty() {
        let address: Address = Address::ZERO;
        let address_ed: AddressED = address.into();
        let bytes = address_ed.encode_vec();
        let decoded = AddressED::decode_vec(&bytes).unwrap();
        assert_eq!(address_ed, decoded);
    }

    #[test]
    fn test_address_ed_serialize() {
        let address: Address = "0x1234567890123456789012345678901234567890"
            .parse()
            .unwrap();
        let address_ed: AddressED = address.into();
        let serialized = serde_json::to_string(&address_ed).unwrap();
        assert_eq!(serialized, "\"0x1234567890123456789012345678901234567890\"");
    }

    #[test]
    fn test_address_ed_deserialize() {
        let serialized = "\"0x1234567890123456789012345678901234567890\"";
        let address_ed: AddressED = serde_json::from_str(serialized).unwrap();
        assert_eq!(
            address_ed.address.to_string(),
            "0x1234567890123456789012345678901234567890"
        );
    }
}
