use std::{borrow::Cow, error::Error};

use heed::{BytesDecode, BytesEncode};
use revm::primitives::{ruint::aliases::U160, Address};
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
        let mut limbs = [0u64; 3];
        for (i, limb) in limbs.iter_mut().enumerate() {
            let start = i * 8;
            let end = start + 8;
            let bytes = &bytes[start..end];
            *limb = u64::from_be_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]);
        }
        Ok(AddressED(Address::from(U160::from_limbs(limbs))))
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
