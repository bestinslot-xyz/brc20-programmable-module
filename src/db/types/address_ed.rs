use std::error::Error;

use alloy::primitives::Address;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::db::types::{Decode, Encode};
use crate::global::INVALID_ADDRESS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Represents a fixed-size byte array of length 20, which is used to represent EVM addresses.
///
/// Wrapper around `Address` to provide serialization and deserialization
/// functionality for EVM addresses in a fixed-size format.
pub struct AddressED {
    /// The address value.
    pub address: Address,
}

impl AddressED {
    /// Creates a new `AddressED` instance from the given address.
    pub fn new(address: Address) -> Self {
        Self { address }
    }
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
        AddressED::new(address)
    }
}

impl TryFrom<&str> for AddressED {
    type Error = Box<dyn Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let address = value.parse::<Address>()?;
        Ok(AddressED::new(address))
    }
}

impl<'de> Deserialize<'de> for AddressED {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_string = String::deserialize(deserializer)?;
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
