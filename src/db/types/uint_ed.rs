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
    fn encode(&self, buffer: &mut Vec<u8>) {
        for limb in self.uint.as_limbs().iter().rev() {
            limb.encode(buffer);
        }
    }
}

impl<const BITS: usize, const LIMBS: usize> Decode for UintED<BITS, LIMBS> {
    fn decode(bytes: &[u8], mut offset: usize) -> Result<(Self, usize), Box<dyn Error>>
    where
        Self: Sized,
    {
        let mut limbs = [0u64; LIMBS];
        for i in 0..LIMBS {
            (limbs[i], offset) = Decode::decode(bytes, offset)?;
        }
        limbs.reverse();
        Ok((
            Self {
                uint: Uint::from_limbs(limbs),
            },
            offset,
        ))
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
        let bytes = u64_ed.encode_vec();
        let decoded = U64ED::decode_vec(&bytes).unwrap();
        assert_eq!(u64_ed, decoded);
    }

    #[test]
    fn test_u256_ed() {
        let u256_ed: U256ED = U256::from(100u64).into();
        let bytes = u256_ed.encode_vec();
        let decoded = U256ED::decode_vec(&bytes).unwrap();
        assert_eq!(u256_ed, decoded);
    }

    #[test]
    fn test_u512_ed() {
        let u512_ed: U512ED = U512::from(100u64).into();
        let bytes = u512_ed.encode_vec();
        let decoded = U512ED::decode_vec(&bytes).unwrap();
        assert_eq!(u512_ed, decoded);
    }

    #[test]
    fn test_u512_ed_from_addr() {
        let u256 = U256::from(1u64);
        let u512_ed = U512ED::from_addr_u256([100u8; 20].into(), u256).unwrap();
        let bytes = u512_ed.encode_vec();
        let decoded = U512ED::decode_vec(&bytes).unwrap();
        assert_eq!(u512_ed, decoded);
    }

    #[test]
    fn test_u512_ed_from_addr_str() {
        let address = "0x1234567890123456789012345678901234567890"
            .parse()
            .unwrap();
        let u256 = U256::from(1u64);
        let u512_ed = U512ED::from_addr_u256(address, u256).unwrap();
        let bytes = u512_ed.encode_vec();
        let decoded = U512ED::decode_vec(&bytes).unwrap();
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
