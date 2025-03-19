use std::error::Error;

pub trait Encode {
    fn encode(&self) -> Result<Vec<u8>, Box<dyn Error>>;
}

pub trait Decode {
    fn decode(bytes: Vec<u8>) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
}
