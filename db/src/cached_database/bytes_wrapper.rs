use std::{borrow::Cow, error::Error};

use heed::{BytesDecode, BytesEncode};

pub(crate) struct BytesWrapper(Vec<u8>);

// Convert from [u8]
impl BytesWrapper {
    pub fn from_vec(bytes: Vec<u8>) -> Self {
        BytesWrapper(bytes)
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.clone()
    }
}

impl<'a> BytesDecode<'a> for BytesWrapper {
    type DItem = BytesWrapper;

    fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem, Box<dyn Error + Send + Sync>> {
        Ok(BytesWrapper(bytes.to_vec()))
    }
}

impl<'a> BytesEncode<'a> for BytesWrapper {
    type EItem = BytesWrapper;

    fn bytes_encode(item: &'a Self::EItem) -> Result<Cow<'a, [u8]>, Box<dyn Error + Send + Sync>> {
        Ok(Cow::Borrowed(&item.0))
    }
}
