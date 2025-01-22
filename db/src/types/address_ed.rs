use std::{borrow::Cow, error::Error};

use heed::{BytesDecode, BytesEncode};
use revm::primitives::Address;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct AddressED(pub Address);

impl AddressED {
    pub fn from_addr(a: Address) -> Self {
        Self(a)
    }
}

impl<'a> BytesEncode<'a> for AddressED {
    type EItem = AddressED;

    fn bytes_encode(item: &'a Self::EItem) -> Result<Cow<'a, [u8]>, Box<dyn Error>> {
        let bytes = item.0 .0.to_vec();
        Ok(Cow::Owned(bytes))
    }
}

impl<'a> BytesDecode<'a> for AddressED {
    type DItem = AddressED;

    fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem, Box<dyn Error>> {
        let mut bytes_array = [0u8; 20];
        bytes_array.copy_from_slice(bytes);
        Ok(AddressED(Address::from(bytes_array)))
    }
}

#[cfg(test)]
mod tests {
    use heed::{BytesDecode, BytesEncode};
    use revm::primitives::Address;

    use crate::types::AddressED;

    #[test]
    fn test_address_ed() {
        let address: Address = "0x1234567890123456789012345678901234567890"
            .parse()
            .unwrap();
        let address_ed = AddressED::from_addr(address);
        let bytes = AddressED::bytes_encode(&address_ed).unwrap();
        let decoded = AddressED::bytes_decode(&bytes).unwrap();
        assert_eq!(address_ed.0, decoded.0);
    }
}
