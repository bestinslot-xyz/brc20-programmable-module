use std::error::Error;
use std::fmt;

use revm::primitives::ruint::aliases::U256;
use revm::primitives::ruint::Uint;
use revm::primitives::Address;
use serde::Serialize;

use crate::db::types::{Decode, Encode};

#[derive(Clone, PartialEq, PartialOrd, Ord, Eq, Hash, Debug)]
pub struct UintEncodeDecode<const BITS: usize, const LIMBS: usize>(pub Uint<BITS, LIMBS>);
pub type U64ED = UintEncodeDecode<64, 1>;
pub type U128ED = UintEncodeDecode<128, 2>;
pub type U512ED = UintEncodeDecode<512, 8>;
pub type U256ED = UintEncodeDecode<256, 4>;

impl U512ED {
    pub fn from_addr_u256(a: Address, b: U256) -> Result<Self, Box<dyn Error>> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&a.0.to_vec());
        bytes.extend_from_slice(&[0u8; 12]);
        bytes.extend_from_slice(&b.to_be_bytes::<32>().to_vec());
        return Ok(Self(Uint::from_be_bytes::<64>(
            bytes.as_slice().try_into()?,
        )));
    }
}

impl U256ED {
    pub fn from_u256(a: U256) -> Self {
        Self(a)
    }
}

impl U128ED {
    pub fn from_u128(a: u128) -> Self {
        Self(Uint::from(a))
    }
}

impl U64ED {
    pub fn from_u64(a: u64) -> Self {
        Self(Uint::from(a))
    }

    pub fn to_u64(&self) -> u64 {
        self.0.as_limbs()[0]
    }
}

impl<const BITS: usize, const LIMBS: usize> Serialize for UintEncodeDecode<BITS, LIMBS> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let hex_string = format!("0x{:x}", self.0);
        serializer.serialize_str(&hex_string)
    }
}

impl<const BITS: usize, const LIMBS: usize> Encode for UintEncodeDecode<BITS, LIMBS> {
    fn encode(&self) -> Vec<u8> {
        let mut limbs = self.0.as_limbs().to_vec();
        limbs.reverse();
        let bytes = limbs
            .iter()
            .flat_map(|limb| limb.to_be_bytes().to_vec())
            .collect::<Vec<u8>>();
        bytes
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

impl<const BITS: usize, const LIMBS: usize> fmt::Display for UintEncodeDecode<BITS, LIMBS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use revm::primitives::ruint::aliases::U256;

    use super::*;

    #[test]
    fn test_u64_ed() {
        let u64_ed = U256ED::from_u256(U256::from(100u64));
        let bytes = u64_ed.encode();
        let decoded = U256ED::decode(bytes).unwrap();
        assert_eq!(u64_ed.0, decoded.0);
    }

    #[test]
    fn test_u256_ed() {
        let u256: U256 = U256::from(100u64);
        let u256_ed = U256ED::from_u256(u256);
        let bytes = u256_ed.encode();
        let decoded = U256ED::decode(bytes).unwrap();
        assert_eq!(u256_ed.0, decoded.0);
    }

    #[test]
    fn test_u512_ed() {
        let u256: U256 = U256::from(100u64);
        let u256_ed = U256ED::from_u256(u256);
        let bytes = u256_ed.encode();
        let decoded = U256ED::decode(bytes).unwrap();
        assert_eq!(u256_ed.0, decoded.0);
    }

    #[test]
    fn test_u512_ed_from_addr() {
        let u256 = U256::from(1u64);
        let u512_ed = U512ED::from_addr_u256([100u8; 20].into(), u256).unwrap();
        let bytes = u512_ed.encode();
        let decoded = U512ED::decode(bytes).unwrap();
        assert_eq!(u512_ed.0, decoded.0);
    }

    #[test]
    fn test_u512_ed_from_addr_str() {
        let address = "0x1234567890123456789012345678901234567890"
            .parse()
            .unwrap();
        let u256 = U256::from(1u64);
        let u512_ed = U512ED::from_addr_u256(address, u256).unwrap();
        let bytes = u512_ed.encode();
        let decoded = U512ED::decode(bytes).unwrap();
        assert_eq!(u512_ed.0, decoded.0);
    }
}
