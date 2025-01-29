use std::error::Error;

use revm::primitives::Address;

use super::{Decode, Encode};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct AddressED(pub Address);

impl AddressED {
    pub fn from_addr(a: Address) -> Self {
        Self(a)
    }
}

impl Encode for AddressED {
    fn encode(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.0.to_vec());
        Ok(bytes)
    }
}

impl Decode for AddressED {
    fn decode(bytes: Vec<u8>) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let mut bytes_array = [0u8; 20];
        bytes_array.copy_from_slice(&bytes);
        Ok(AddressED(Address::from(bytes_array)))
    }
}

#[cfg(test)]
mod tests {
    use revm::primitives::Address;

    use crate::types::{AddressED, Decode, Encode};

    #[test]
    fn test_address_ed() {
        let address: Address = "0x1234567890123456789012345678901234567890"
            .parse()
            .unwrap();
        let address_ed = AddressED::from_addr(address);
        let bytes = AddressED::encode(&address_ed).unwrap();
        let decoded = AddressED::decode(bytes).unwrap();
        assert_eq!(address_ed.0, decoded.0);
    }
}
