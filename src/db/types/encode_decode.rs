use std::error::Error;

pub trait Encode {
    fn encode(&self, buffer: &mut Vec<u8>);

    fn encode_vec(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        self.encode(&mut buffer);
        buffer
    }
}

pub trait Decode {
    fn decode(bytes: &[u8], offset: usize) -> Result<(Self, usize), Box<dyn Error>>
    where
        Self: Sized;

    fn decode_vec(bytes: &Vec<u8>) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        Ok(Self::decode(bytes.as_slice(), 0)?.0)
    }
}

impl<T> Encode for &T
where
    T: Encode,
{
    fn encode(&self, buffer: &mut Vec<u8>) {
        (*self).encode(buffer);
    }
}

impl Encode for String {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.as_bytes().to_vec().encode(buffer);
    }
}

impl Decode for String {
    fn decode(bytes: &[u8], offset: usize) -> Result<(Self, usize), Box<dyn Error>> {
        let (bytes, offset) = Vec::<u8>::decode(bytes, offset)?;
        Ok((
            String::from_utf8(bytes).map_err(|_| "Invalid UTF-8")?,
            offset,
        ))
    }
}

impl Encode for &str {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.as_bytes().to_vec().encode(buffer);
    }
}

impl<T> Encode for Vec<T>
where
    T: Encode,
{
    fn encode(&self, buffer: &mut Vec<u8>) {
        (self.len() as u32).encode(buffer);
        for item in self {
            item.encode(buffer);
        }
    }
}

impl<T> Decode for Vec<T>
where
    T: Decode,
{
    fn decode(bytes: &[u8], offset: usize) -> Result<(Self, usize), Box<dyn Error>> {
        let (length, offset) = u32::decode(bytes, offset)?;
        let mut vec = Vec::with_capacity(length as usize);
        let mut current_offset = offset;
        for _ in 0..length {
            let (item, next_offset) = T::decode(bytes, current_offset)?;
            vec.push(item);
            current_offset = next_offset;
        }
        Ok((vec, current_offset))
    }
}

impl<T, const N: usize> Encode for [T; N]
where
    T: Encode,
{
    fn encode(&self, buffer: &mut Vec<u8>) {
        for item in self {
            item.encode(buffer);
        }
    }
}

impl<T, const N: usize> Decode for [T; N]
where
    T: Decode,
{
    fn decode(bytes: &[u8], offset: usize) -> Result<(Self, usize), Box<dyn Error>> {
        let mut array = Vec::with_capacity(N);
        let mut current_offset = offset;
        for _ in 0..N {
            let (item, next_offset) = T::decode(bytes, current_offset)?;
            array.push(item);
            current_offset = next_offset;
        }
        Ok((
            array.try_into().map_err(|_| "Invalid array length")?,
            current_offset,
        ))
    }
}

impl<T> Encode for Option<T>
where
    T: Encode,
{
    fn encode(&self, buffer: &mut Vec<u8>) {
        match self {
            Some(value) => {
                buffer.push(1);
                value.encode(buffer);
            }
            None => {
                buffer.push(0);
            }
        }
    }
}

impl<T> Decode for Option<T>
where
    T: Decode,
{
    fn decode(bytes: &[u8], offset: usize) -> Result<(Self, usize), Box<dyn Error>> {
        // First byte indicates presence
        let (flag, offset) = u8::decode(bytes, offset)?;
        if flag == 1 {
            let (value, offset) = T::decode(bytes, offset)?;
            Ok((Some(value), offset))
        } else {
            Ok((None, offset))
        }
    }
}

impl Encode for u64 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.to_be_bytes().encode(buffer);
    }
}

impl Decode for u64 {
    fn decode(bytes: &[u8], offset: usize) -> Result<(Self, usize), Box<dyn Error>> {
        <[u8; 8]>::decode(bytes, offset)
            .map(|(be_bytes, offset)| (u64::from_be_bytes(be_bytes), offset))
    }
}

impl Encode for u32 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.to_be_bytes().encode(buffer);
    }
}

impl Decode for u32 {
    fn decode(bytes: &[u8], offset: usize) -> Result<(Self, usize), Box<dyn Error>> {
        <[u8; 4]>::decode(bytes, offset)
            .map(|(be_bytes, offset)| (u32::from_be_bytes(be_bytes), offset))
    }
}

impl Encode for u8 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.push(*self);
    }
}

impl Decode for u8 {
    fn decode(bytes: &[u8], offset: usize) -> Result<(Self, usize), Box<dyn Error>> {
        Ok((bytes[offset], offset + 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let original = "Hello, world!";
        let mut buffer = Vec::new();
        original.encode(&mut buffer);
        let (decoded, _) = String::decode(&buffer, 0).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_vec_encode_decode() {
        let original = vec![1, 2, 3, 4, 5];
        let mut buffer = Vec::new();
        original.encode(&mut buffer);
        let (decoded, _) = Vec::<u8>::decode(&buffer, 0).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_array_encode_decode() {
        let original = [1, 2, 3, 4, 5];
        let mut buffer = Vec::new();
        original.encode(&mut buffer);
        let (decoded, _) = <[u8; 5]>::decode(&buffer, 0).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_option_encode_decode() {
        let original = Some(42);
        let mut buffer = Vec::new();
        original.encode(&mut buffer);
        let (decoded, _) = Option::<u32>::decode(&buffer, 0).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_option_none_encode_decode() {
        let original: Option<u32> = None;
        let mut buffer = Vec::new();
        original.encode(&mut buffer);
        let (decoded, _) = Option::<u32>::decode(&buffer, 0).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_u64_encode_decode() {
        let original: u64 = 1234567890123456789;
        let mut buffer = Vec::new();
        original.encode(&mut buffer);
        let (decoded, _) = u64::decode(&buffer, 0).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_u32_encode_decode() {
        let original: u32 = 1234567890;
        let mut buffer = Vec::new();
        original.encode(&mut buffer);
        let (decoded, _) = u32::decode(&buffer, 0).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_u8_encode_decode() {
        let original: u8 = 42;
        let mut buffer = Vec::new();
        original.encode(&mut buffer);
        let (decoded, _) = u8::decode(&buffer, 0).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_bytes_encode_decode() {
        let original: Vec<u8> = vec![1, 2, 3, 4, 5];
        let mut buffer = Vec::new();
        original.encode(&mut buffer);
        let (decoded, _) = Vec::<u8>::decode(&buffer, 0).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_bytes_array_encode_decode() {
        let original: [u8; 5] = [1, 2, 3, 4, 5];
        let mut buffer = Vec::new();
        original.encode(&mut buffer);
        let (decoded, _) = <[u8; 5]>::decode(&buffer, 0).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_bytes_option_encode_decode() {
        let original: Option<Vec<u8>> = Some(vec![1, 2, 3, 4, 5]);
        let mut buffer = Vec::new();
        original.encode(&mut buffer);
        let (decoded, _) = Option::<Vec<u8>>::decode(&buffer, 0).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_bytes_option_none_encode_decode() {
        let original: Option<Vec<u8>> = None;
        let mut buffer = Vec::new();
        original.encode(&mut buffer);
        let (decoded, _) = Option::<Vec<u8>>::decode(&buffer, 0).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_bytes_array_option_encode_decode() {
        let original: Option<[u8; 5]> = Some([1, 2, 3, 4, 5]);
        let mut buffer = Vec::new();
        original.encode(&mut buffer);
        let (decoded, _) = Option::<[u8; 5]>::decode(&buffer, 0).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_bytes_array_option_none_encode_decode() {
        let original: Option<[u8; 5]> = None;
        let mut buffer = Vec::new();
        original.encode(&mut buffer);
        let (decoded, _) = Option::<[u8; 5]>::decode(&buffer, 0).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_bytes_option_array_encode_decode() {
        let original: Option<Vec<[u8; 5]>> = Some(vec![[1, 2, 3, 4, 5], [6, 7, 8, 9, 10]]);
        let mut buffer = Vec::new();
        original.encode(&mut buffer);
        let (decoded, _) = Option::<Vec<[u8; 5]>>::decode(&buffer, 0).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_bytes_option_array_none_encode_decode() {
        let original: Option<Vec<[u8; 5]>> = None;
        let mut buffer = Vec::new();
        original.encode(&mut buffer);
        let (decoded, _) = Option::<Vec<[u8; 5]>>::decode(&buffer, 0).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_bytes_array_option_array_encode_decode() {
        let original: Option<[Vec<u8>; 2]> = Some([vec![1, 2, 3], vec![4, 5, 6]]);
        let mut buffer = Vec::new();
        original.encode(&mut buffer);
        let (decoded, _) = Option::<[Vec<u8>; 2]>::decode(&buffer, 0).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_bytes_array_option_array_none_encode_decode() {
        let original: Option<[Vec<u8>; 2]> = None;
        let mut buffer = Vec::new();
        original.encode(&mut buffer);
        let (decoded, _) = Option::<[Vec<u8>; 2]>::decode(&buffer, 0).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_bytes_option_array_option_encode_decode() {
        let original: Option<Vec<Option<[u8; 5]>>> = Some(vec![Some([1, 2, 3, 4, 5]), None]);
        let mut buffer = Vec::new();
        original.encode(&mut buffer);
        let (decoded, _) = Option::<Vec<Option<[u8; 5]>>>::decode(&buffer, 0).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_bytes_option_array_option_none_encode_decode() {
        let original: Option<Vec<Option<[u8; 5]>>> = None;
        let mut buffer = Vec::new();
        original.encode(&mut buffer);
        let (decoded, _) = Option::<Vec<Option<[u8; 5]>>>::decode(&buffer, 0).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_bytes_array_option_array_option_encode_decode() {
        let original: Option<[Option<Vec<u8>>; 2]> = Some([Some(vec![1, 2, 3]), None]);
        let mut buffer = Vec::new();
        original.encode(&mut buffer);
        let (decoded, _) = Option::<[Option<Vec<u8>>; 2]>::decode(&buffer, 0).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_bytes_array_option_array_option_none_encode_decode() {
        let original: Option<[Option<Vec<u8>>; 2]> = None;
        let mut buffer = Vec::new();
        original.encode(&mut buffer);
        let (decoded, _) = Option::<[Option<Vec<u8>>; 2]>::decode(&buffer, 0).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_bytes_option_array_option_array_encode_decode() {
        let original: Option<Vec<Option<[u8; 5]>>> = Some(vec![Some([1, 2, 3, 4, 5]), None]);
        let mut buffer = Vec::new();
        original.encode(&mut buffer);
        let (decoded, _) = Option::<Vec<Option<[u8; 5]>>>::decode(&buffer, 0).unwrap();
        assert_eq!(original, decoded);
    }
}
