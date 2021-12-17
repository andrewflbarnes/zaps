use hex::FromHexError;
use std::convert::From;

#[derive(Debug)]
pub enum DecodeBitmapError {
    InvalidChar{
        c: char, index: usize,
    },
    InvalidStringLength,
    OddLength,
}

impl From<FromHexError> for DecodeBitmapError {
    fn from(hex_error: FromHexError) -> Self {
        match hex_error {
            FromHexError::InvalidHexCharacter{ c, index } => DecodeBitmapError::InvalidChar{c, index},
            FromHexError::InvalidStringLength => DecodeBitmapError::InvalidStringLength,
            FromHexError::OddLength => DecodeBitmapError::OddLength,
        }
    }
}

pub fn decode_ascii_bitmap(raw_bitmap: &[u8], size: usize) -> Result<u64, DecodeBitmapError> {
    let mut decoded = vec![0u8; size / 2];
    hex::decode_to_slice(raw_bitmap, &mut decoded as &mut [u8])?;

    decode_bitmap(&decoded, size / 2)
}

pub fn decode_bitmap(raw_bitmap: &[u8], size: usize) -> Result<u64, DecodeBitmapError> {
    let mut bitmap = 0u64;

    if raw_bitmap.len() < size {
        return Err(DecodeBitmapError::InvalidStringLength);
    }

    if raw_bitmap.len() % 2 == 1 {
        return Err(DecodeBitmapError::OddLength);
    }

    for (i, byte) in raw_bitmap.iter().enumerate().take(size) {
        for j in 0..8 {
            let bit = 7 - j;
            if 1 & (byte >> bit) == 1 {
                let shift = 8 * (7 - i) + 7 - j;
                bitmap += 1 << shift;
            }
        }
    }
    Ok(bitmap)
}

#[cfg(test)]
mod test {
    use super::{
        decode_ascii_bitmap,
        decode_bitmap,
    };

    #[test]
    fn test_ascii_bitmap() {
        let raw_bitmap = "8080000000400001".as_bytes();
        let bitmap = decode_ascii_bitmap(&raw_bitmap, raw_bitmap.len()).unwrap();

        for i in 0..64 {
            let bitpos = 63 - i;
            match i {
                0 | 8 | 41 | 63 => assert!(1 & (bitmap >> bitpos) == 1),
                _ => assert!(1 & (bitmap >> bitpos) == 0),
            }
        }

    }

    #[test]
    fn test_bitmap() {
        let raw_bitmap = [
            0x80, 0x80, 0x00, 0x00, 0x00, 0x40, 0x00, 0x01,
        ];
        let bitmap = decode_bitmap(&raw_bitmap, raw_bitmap.len()).unwrap();

        for i in 0..64 {
            let bitpos = 63 - i;
            match i {
                0 | 8 | 41 | 63 => assert!(1 & (bitmap >> bitpos) == 1),
                _ => assert!(1 & (bitmap >> bitpos) == 0),
            }
        }

    }
}