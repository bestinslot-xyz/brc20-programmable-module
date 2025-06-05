use std::error::Error;
use std::fmt;

use alloy_primitives::Uint;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::db::types::{Decode, Encode};

#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Debug)]
/// Represents an unsigned integer with a fixed number of bits and limbs.
///
/// Wrapper around `Uint` to provide serialization and deserialization
/// functionality for unsigned integers in a fixed-size format.
pub struct UintED<const BITS: usize, const LIMBS: usize> {
    /// The Uint value.
    pub uint: Uint<BITS, LIMBS>,
}

/// Type alias for a 8-bit unsigned integer with 1 limb
pub type U8ED = UintED<8, 1>;
/// Type alias for a 64-bit unsigned integer with 1 limb
pub type U64ED = UintED<64, 1>;
/// Type alias for a 128-bit unsigned integer with 2 limbs
pub type U128ED = UintED<128, 2>;
/// Type alias for a 512-bit unsigned integer with 4 limbs
pub type U512ED = UintED<512, 8>;
/// Type alias for a 256-bit unsigned integer with 4 limbs
pub type U256ED = UintED<256, 4>;

impl<const BITS: usize, const LIMBS: usize> UintED<BITS, LIMBS> {
    /// Creates a new `UintED` instance from a `Uint<BITS, LIMBS>` instance.
    pub fn new(uint: Uint<BITS, LIMBS>) -> Self {
        Self { uint }
    }

    /// Returns if the `UintED` instance is zero.
    pub fn is_zero(&self) -> bool {
        self.uint.is_zero()
    }
}

impl<const BITS: usize, const LIMBS: usize> From<Uint<BITS, LIMBS>> for UintED<BITS, LIMBS> {
    fn from(uint: Uint<BITS, LIMBS>) -> Self {
        UintED::new(uint)
    }
}

impl<const BITS: usize, const LIMBS: usize> From<u128> for UintED<BITS, LIMBS> {
    fn from(value: u128) -> Self {
        UintED::new(Uint::from(value))
    }
}

impl<const BITS: usize, const LIMBS: usize> From<u64> for UintED<BITS, LIMBS> {
    fn from(value: u64) -> Self {
        UintED::new(Uint::from(value))
    }
}

impl<const BITS: usize, const LIMBS: usize> From<u32> for UintED<BITS, LIMBS> {
    fn from(value: u32) -> Self {
        UintED::new(Uint::from(value))
    }
}

impl<const BITS: usize, const LIMBS: usize> From<u8> for UintED<BITS, LIMBS> {
    fn from(value: u8) -> Self {
        UintED::new(Uint::from(value))
    }
}

impl Into<u64> for U64ED {
    fn into(self) -> u64 {
        self.uint.as_limbs()[0]
    }
}

#[cfg(feature = "server")]
use alloy_primitives::{Address, U256};

#[cfg(feature = "server")]
impl U512ED {
    // This is used by the server, so doesn't need to be public
    pub(crate) fn from_addr_u256(address: Address, mem_loc: U256) -> Result<Self, Box<dyn Error>> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&address.0.to_vec());
        bytes.extend_from_slice(&[0u8; 12]);
        bytes.extend_from_slice(&mem_loc.to_be_bytes::<32>().to_vec());
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
        S: Serializer,
    {
        let hex_string = format!("0x{:x}", self.uint);
        serializer.serialize_str(&hex_string)
    }
}

impl<'de, const BITS: usize, const LIMBS: usize> Deserialize<'de> for UintED<BITS, LIMBS> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_string = String::deserialize(deserializer)?;
        let mut hex_string = hex_string.trim_start_matches("0x").to_string();
        if hex_string.len() % 2 != 0 {
            hex_string = format!("0{}", hex_string);
        }
        let bytes = hex::decode(hex_string).map_err(serde::de::Error::custom)?;
        let uint = Uint::<BITS, LIMBS>::try_from_be_slice(bytes.as_slice());
        match uint {
            Some(uint) => Ok(Self { uint }),
            None => {
                return Err(serde::de::Error::custom(
                    "Failed to decode integer from hex string",
                ))
            }
        }
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

        let deserialized: U256ED = serde_json::from_str(&serialized).unwrap();
        assert_eq!(u256_ed, deserialized);
    }

    #[test]
    fn test_u512_ed_deserialize() {
        let deserialized: U512ED = serde_json::from_str("\"0x000064\"").unwrap();
        assert_eq!(deserialized.uint.as_limbs()[0], 100u64);
    }

    #[test]
    fn test_u512_ed_serialize() {
        let u512_ed: U512ED = U512::from(100u64).into();
        let serialized = serde_json::to_string(&u512_ed).unwrap();
        assert_eq!(serialized, "\"0x64\"");

        let deserialized: U512ED = serde_json::from_str(&serialized).unwrap();
        assert_eq!(u512_ed, deserialized);
    }
}
