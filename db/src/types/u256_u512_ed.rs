use heed::{BytesDecode, BytesEncode};
use revm::primitives::{
    ruint::{aliases::U256, Uint},
    Address,
};
use std::{borrow::Cow, error::Error, fmt};

use super::{Decode, Encode};

#[derive(Clone, PartialEq, PartialOrd, Ord, Eq, Hash, Debug)]
pub struct UintEncodeDecode<const BITS: usize, const LIMBS: usize>(pub Uint<BITS, LIMBS>);
pub type U512ED = UintEncodeDecode<512, 8>;
pub type U256ED = UintEncodeDecode<256, 4>;

impl U512ED {
    pub fn from_addr_u256(a: Address, b: U256) -> Self {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&a.0.to_vec());
        bytes.extend_from_slice(&[0u8; 12]);
        bytes.extend_from_slice(&b.to_be_bytes::<32>().to_vec());
        return Self(Uint::from_be_bytes::<64>(
            bytes.as_slice().try_into().unwrap(),
        ));
    }
}

impl U256ED {
    pub fn from_u256(a: U256) -> Self {
        Self(a)
    }
}

impl<const BITS: usize, const LIMBS: usize> Encode for UintEncodeDecode<BITS, LIMBS> {
    fn encode(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut limbs = self.0.as_limbs().to_vec();
        limbs.reverse();
        let bytes = limbs
            .iter()
            .flat_map(|limb| limb.to_be_bytes().to_vec())
            .collect::<Vec<u8>>();
        Ok(bytes)
    }
}

impl<const BITS: usize, const LIMBS: usize> Decode for UintEncodeDecode<BITS, LIMBS> {
    fn decode(bytes: Vec<u8>) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let mut limbs = [0u64; LIMBS];
        for (i, limb) in limbs.iter_mut().enumerate() {
            let start = (LIMBS - 1 - i) * 8;
            let end = start + 8;
            let bytes = &bytes[start..end];
            *limb = u64::from_be_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]);
        }
        Ok(UintEncodeDecode(Uint::from_limbs(limbs)))
    }
}

impl<'a, const BITS: usize, const LIMBS: usize> BytesEncode<'a> for UintEncodeDecode<BITS, LIMBS> {
    type EItem = UintEncodeDecode<BITS, LIMBS>;

    fn bytes_encode(item: &'a Self::EItem) -> Result<Cow<'a, [u8]>, Box<dyn Error>> {
        let mut limbs = item.0.as_limbs().to_vec();
        limbs.reverse();
        let bytes = limbs
            .iter()
            .flat_map(|limb| limb.to_be_bytes().to_vec())
            .collect::<Vec<u8>>();
        Ok(Cow::Owned(bytes))
    }
}

impl<'a, const BITS: usize, const LIMBS: usize> BytesDecode<'a> for UintEncodeDecode<BITS, LIMBS> {
    type DItem = UintEncodeDecode<BITS, LIMBS>;

    fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem, Box<dyn Error>> {
        let mut limbs = [0u64; LIMBS];
        for (i, limb) in limbs.iter_mut().enumerate() {
            let start = (LIMBS - 1 - i) * 8;
            let end = start + 8;
            let bytes = &bytes[start..end];
            *limb = u64::from_be_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]);
        }
        Ok(UintEncodeDecode(Uint::from_limbs(limbs)))
    }
}

impl<const BITS: usize, const LIMBS: usize> fmt::Display for UintEncodeDecode<BITS, LIMBS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use revm::primitives::{ruint::aliases::U256, Address};

    use crate::types::{Decode, Encode, U256ED, U512ED};

    #[test]
    fn test_u256_ed() {
        let u256: U256 = U256::from(0u64);
        let u256_ed = U256ED::from_u256(u256);
        let bytes = U256ED::encode(&u256_ed).unwrap();
        let decoded = U256ED::decode(bytes).unwrap();
        assert_eq!(u256_ed.0, decoded.0);
    }

    #[test]
    fn test_u512_ed() {
        let u256: U256 = U256::from(0u64);
        let u256_ed = U256ED::from_u256(u256);
        let bytes = U256ED::encode(&u256_ed).unwrap();
        let decoded = U256ED::decode(bytes).unwrap();
        assert_eq!(u256_ed.0, decoded.0);
    }

    #[test]
    fn test_u512_ed_from_addr() {
        let address = Address::from([0u8; 20]);
        let u256 = U256::from(1u64);
        let u512_ed = U512ED::from_addr_u256(address, u256);
        let bytes = U512ED::encode(&u512_ed).unwrap();
        let decoded = U512ED::decode(bytes).unwrap();
        assert_eq!(u512_ed.0, decoded.0);
    }

    #[test]
    fn test_u512_ed_from_addr_str() {
        let address = Address::from_str("0x1234567890123456789012345678901234567890").unwrap();
        let u256 = U256::from(1u64);
        let u512_ed = U512ED::from_addr_u256(address, u256);
        let bytes = U512ED::encode(&u512_ed).unwrap();
        let decoded = U512ED::decode(bytes).unwrap();
        assert_eq!(u512_ed.0, decoded.0);
    }
}
