use std::error::Error;
use std::fmt;

use alloy_primitives::{Address, Uint, U256};
use serde::{Serialize, Serializer};

use crate::db::types::{Decode, Encode};

#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Debug)]
pub struct UintED<const BITS: usize, const LIMBS: usize> {
    pub uint: Uint<BITS, LIMBS>,
}

pub type U64ED = UintED<64, 1>;
pub type U128ED = UintED<128, 2>;
pub type U512ED = UintED<512, 8>;
pub type U256ED = UintED<256, 4>;

impl<const BITS: usize, const LIMBS: usize> From<Uint<BITS, LIMBS>> for UintED<BITS, LIMBS> {
    fn from(uint: Uint<BITS, LIMBS>) -> Self {
        Self { uint }
    }
}

impl<const BITS: usize, const LIMBS: usize> From<u64> for UintED<BITS, LIMBS> {
    fn from(value: u64) -> Self {
        Self {
            uint: Uint::from(value),
        }
    }
}

impl<const BITS: usize, const LIMBS: usize> From<u128> for UintED<BITS, LIMBS> {
    fn from(value: u128) -> Self {
        Self {
            uint: Uint::from(value),
        }
    }
}

impl<const BITS: usize, const LIMBS: usize> From<u32> for UintED<BITS, LIMBS> {
    fn from(value: u32) -> Self {
        Self {
            uint: Uint::from(value),
        }
    }
}

impl Into<u64> for U64ED {
    fn into(self) -> u64 {
        self.uint.as_limbs()[0]
    }
}

impl U512ED {
    pub fn from_addr_u256(a: Address, b: U256) -> Result<Self, Box<dyn Error>> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&a.0.to_vec());
        bytes.extend_from_slice(&[0u8; 12]);
        bytes.extend_from_slice(&b.to_be_bytes::<32>().to_vec());
        Ok(Self {
            uint: Uint::from_be_bytes::<64>(bytes.as_slice().try_into()?),
        })
    }
}

pub fn uint_full_hex<const BITS: usize, const LIMBS: usize, S: Serializer>(
    number: &UintED<BITS, LIMBS>,
    s: S,
) -> Result<S::Ok, S::Error> {
    let hex_string = format!("{:x}", number.uint);
    let padding = BITS / 4 - hex_string.len();
    let hex_string = format!(
        "{:0>width$}",
        hex_string,
        width = padding + hex_string.len()
    );
    let hex_string = format!("0x{}", hex_string);
    s.serialize_str(&hex_string)
}

impl<const BITS: usize, const LIMBS: usize> Serialize for UintED<BITS, LIMBS> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let hex_string = format!("0x{:x}", self.uint);
        serializer.serialize_str(&hex_string)
    }
}

impl<const BITS: usize, const LIMBS: usize> Encode for UintED<BITS, LIMBS> {
    fn encode(&self) -> Vec<u8> {
        let mut limbs = self.uint.as_limbs().to_vec();
        limbs.reverse();
        let bytes = limbs
            .iter()
            .flat_map(|limb| limb.to_be_bytes().to_vec())
            .collect::<Vec<u8>>();
        bytes
    }
}

impl<const BITS: usize, const LIMBS: usize> Decode for UintED<BITS, LIMBS> {
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
        Ok(Self {
            uint: Uint::from_limbs(limbs),
        })
    }
}

impl<const BITS: usize, const LIMBS: usize> fmt::Display for UintED<BITS, LIMBS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.uint)
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::U512;

    use super::*;

    #[test]
    fn test_u64_ed() {
        let u64_ed: U64ED = 100u64.into();
        let bytes = u64_ed.encode();
        let decoded = U64ED::decode(bytes).unwrap();
        assert_eq!(u64_ed, decoded);
    }

    #[test]
    fn test_u256_ed() {
        let u256_ed: U256ED = U256::from(100u64).into();
        let bytes = u256_ed.encode();
        let decoded = U256ED::decode(bytes).unwrap();
        assert_eq!(u256_ed, decoded);
    }

    #[test]
    fn test_u512_ed() {
        let u512_ed: U512ED = U512::from(100u64).into();
        let bytes = u512_ed.encode();
        let decoded = U512ED::decode(bytes).unwrap();
        assert_eq!(u512_ed, decoded);
    }

    #[test]
    fn test_u512_ed_from_addr() {
        let u256 = U256::from(1u64);
        let u512_ed = U512ED::from_addr_u256([100u8; 20].into(), u256).unwrap();
        let bytes = u512_ed.encode();
        let decoded = U512ED::decode(bytes).unwrap();
        assert_eq!(u512_ed, decoded);
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
        assert_eq!(u512_ed, decoded);
    }

    #[test]
    fn test_u64_ed_serialize() {
        let u64_ed: U64ED = 100u64.into();
        let serialized = serde_json::to_string(&u64_ed).unwrap();
        assert_eq!(serialized, "\"0x64\"");
    }
    #[test]

    fn test_u256_ed_serialize() {
        let u256_ed: U256ED = U256::from(100u64).into();
        let serialized = serde_json::to_string(&u256_ed).unwrap();
        assert_eq!(serialized, "\"0x64\"");
    }
}
