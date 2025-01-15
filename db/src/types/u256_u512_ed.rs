use heed::{BytesDecode, BytesEncode};
use revm::primitives::{
    ruint::{
        aliases::{U160, U256},
        Uint,
    },
    Address,
};
use std::{borrow::Cow, error::Error, fmt};

#[derive(Clone)]
pub struct UintEncodeDecode<const BITS: usize, const LIMBS: usize>(pub Uint<BITS, LIMBS>);
pub type U512ED = UintEncodeDecode<512, 8>;
pub type U256ED = UintEncodeDecode<256, 4>;

impl U512ED {
    pub fn from_addr_u256(a: Address, b: U256) -> Self {
        let addr_u160 = U160::from_be_bytes(a.0 .0);
        let limbs1 = addr_u160.as_limbs();
        let limbs2 = b.as_limbs();
        let mut limbs = [0u64; 8];
        for i in 0..4 {
            limbs[i + 0] = limbs2[i];
        }
        for i in 0..3 {
            limbs[i + 4] = u64::from(limbs1[i]);
        }

        Self(Uint::from_limbs(limbs))
    }
}
impl U256ED {
    pub fn from_u256(a: U256) -> Self {
        Self(a)
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
    use heed::{BytesDecode, BytesEncode};
    use revm::primitives::ruint::aliases::U256;

    use crate::types::U256ED;

    #[test]
    fn test_u256_ed() {
        let u256: U256 = U256::from(0u64);
        let u256_ed = U256ED::from_u256(u256);
        let bytes = U256ED::bytes_encode(&u256_ed).unwrap();
        let decoded = U256ED::bytes_decode(&bytes).unwrap();
        assert_eq!(u256_ed.0, decoded.0);
    }

    #[test]
    fn test_u512_ed() {
        let u256: U256 = U256::from(0u64);
        let u256_ed = U256ED::from_u256(u256);
        let bytes = U256ED::bytes_encode(&u256_ed).unwrap();
        let decoded = U256ED::bytes_decode(&bytes).unwrap();
        assert_eq!(u256_ed.0, decoded.0);
    }
}
