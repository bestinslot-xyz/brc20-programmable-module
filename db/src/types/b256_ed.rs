use heed::{BytesDecode, BytesEncode};
use revm::primitives::{FixedBytes, B256};
use std::{borrow::Cow, error::Error};

pub struct BEncodeDecode<const N: usize>(pub FixedBytes<N>);
pub type B256ED = BEncodeDecode<32>;

impl B256ED {
    pub fn from_b256(a: B256) -> Self {
        Self(a)
    }
}

impl<'a, const N: usize> BytesEncode<'a> for BEncodeDecode<N> {
    type EItem = BEncodeDecode<N>;

    fn bytes_encode(item: &'a Self::EItem) -> Result<Cow<'a, [u8]>, Box<dyn Error>> {
        let bytes = item.0.as_slice().to_vec();
        Ok(Cow::Owned(bytes))
    }
}
impl<'a, const N: usize> BytesDecode<'a> for BEncodeDecode<N> {
    type DItem = BEncodeDecode<N>;

    fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem, Box<dyn Error>> {
        let mut arr = [0u8; N];
        arr.copy_from_slice(&bytes);
        Ok(BEncodeDecode(FixedBytes::from(arr)))
    }
}

#[cfg(test)]
mod tests {
    use heed::{BytesDecode, BytesEncode};
    use revm::primitives::B256;

    use crate::types::B256ED;

    #[test]
    fn test_b256_ed() {
        let b256: B256 = B256::from([0u8; 32]);
        let b256_ed = B256ED::from_b256(b256);
        let bytes = B256ED::bytes_encode(&b256_ed).unwrap();
        let decoded = B256ED::bytes_decode(&bytes).unwrap();
        assert_eq!(b256_ed.0, decoded.0);
    }
}
