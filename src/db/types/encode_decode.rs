use std::error::Error;

pub trait Encode {
    fn encode(&self) -> Vec<u8>;
}

pub trait Decode {
    fn decode(bytes: Vec<u8>) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
}

impl Encode for String {
    fn encode(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl Decode for String {
    fn decode(bytes: Vec<u8>) -> Result<Self, Box<dyn Error>> {
        Ok(String::from_utf8(bytes)?)
    }
}
