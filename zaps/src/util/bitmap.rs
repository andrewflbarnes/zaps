use hex::FromHexError;
use std::error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum DecodeBitmapError {
    BitmapTooLarge{
        actual: usize,
        expected: usize,
    },
    BitmapTooShort{
        actual: usize,
        expected: usize,
    },
    InvalidChar{
        c: char, index: usize,
    },
    OddLength,
}

impl fmt::Display for DecodeBitmapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BitmapTooLarge{ actual, expected } |
            Self::BitmapTooShort{ actual, expected } => write!(f, "bitmap was {} bytes but expected {}", actual, expected),
            Self::InvalidChar{ c, index } => write!(f, "invalid char {} at index {} for bitmap", c, index),
            Self::OddLength => write!(f, "odd string length not allowed for bitmap"),
        }
    }
}

impl error::Error for DecodeBitmapError {}

impl DecodeBitmapError {
    fn from(hex_error: FromHexError, actual: usize, expected: usize) -> Self {
        match hex_error {
            FromHexError::InvalidHexCharacter{ c, index } => DecodeBitmapError::InvalidChar{c, index},
            FromHexError::InvalidStringLength => if actual < expected {
                DecodeBitmapError::BitmapTooShort{actual, expected}
            } else {
                DecodeBitmapError::BitmapTooLarge{actual, expected}
            },
            FromHexError::OddLength => DecodeBitmapError::OddLength,
        }
    }
}

pub fn decode_ascii_bitmap(raw_bitmap: &[u8], size: usize) -> Result<u64, DecodeBitmapError> {
    let byte_size = size / 2;
    let mut decoded = vec![0u8; byte_size];
    // convert from oacked ascii hex to raw bytes
    hex::decode_to_slice(raw_bitmap, &mut decoded as &mut [u8])
        .map_err(|e| DecodeBitmapError::from(e, raw_bitmap.len(), size))?;

    decode_bitmap(&decoded, byte_size)
}

pub fn decode_bitmap(raw_bitmap: &[u8], size: usize) -> Result<u64, DecodeBitmapError> {
    let mut bitmap = 0u64;
    let bitmap_len = raw_bitmap.len();

    if bitmap_len != size {
        let decode_err = if bitmap_len < size {
            DecodeBitmapError::BitmapTooShort{
                actual: bitmap_len,
                expected: size
            }
        } else {
            DecodeBitmapError::BitmapTooLarge{
                actual: bitmap_len,
                expected: size
            }
        };
        return Err(decode_err);
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
        DecodeBitmapError,
    };

    mod decode_ascii_bitmap {
        use super::*;

        #[test]
        fn standard_size() {
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
        fn minimum_size() {
            let raw_bitmap = "82".as_bytes();
            let bitmap = decode_ascii_bitmap(&raw_bitmap, raw_bitmap.len()).unwrap();

            for i in 0..64 {
                let bitpos = 63 - i;
                match i {
                    0 | 6 => assert!(1 & (bitmap >> bitpos) == 1),
                    _ => assert!(1 & (bitmap >> bitpos) == 0),
                }
            }
        }

        #[test]
        fn odd_size_err() {
            let raw_bitmap = "820".as_bytes();
            let bitmap = decode_ascii_bitmap(&raw_bitmap, raw_bitmap.len());

            assert_eq!(Err(DecodeBitmapError::OddLength), bitmap);
        }

        #[test]
        fn too_small_err() {
            let raw_bitmap = "82".as_bytes();
            let bitmap = decode_ascii_bitmap(&raw_bitmap, 4);

            assert_eq!(Err(DecodeBitmapError::BitmapTooShort{
                actual: 2,
                expected: 4,
            }), bitmap);
        }

        #[test]
        fn too_large_err() {
            let raw_bitmap = "8200".as_bytes();
            let bitmap = decode_ascii_bitmap(&raw_bitmap, 2);

            assert_eq!(Err(DecodeBitmapError::BitmapTooLarge{
                actual: 4,
                expected: 2,
            }), bitmap);
        }
    }

    mod decode_bitmap {
        use super::*;

        #[test]
        fn standard_size() {
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

        #[test]
        fn minimum_size() {
            let raw_bitmap = [
                0x82
            ];
            let bitmap = decode_bitmap(&raw_bitmap, raw_bitmap.len()).unwrap();

            for i in 0..64 {
                let bitpos = 63 - i;
                match i {
                    0 | 6 => assert!(1 & (bitmap >> bitpos) == 1),
                    _ => assert!(1 & (bitmap >> bitpos) == 0),
                }
            }
        }

        #[test]
        fn too_large_err() {
            let raw_bitmap = [
                0x82, 0x00,
            ];
            let bitmap = decode_bitmap(&raw_bitmap, 1);

            assert_eq!(Err(DecodeBitmapError::BitmapTooLarge{
                actual: 2,
                expected: 1,
            }), bitmap);
        }

        #[test]
        fn too_small_err() {
            let raw_bitmap = [
                0x82, 0x00,
            ];
            let bitmap = decode_bitmap(&raw_bitmap, 3);

            assert_eq!(Err(DecodeBitmapError::BitmapTooShort{
                actual: 2,
                expected: 3,
            }), bitmap);
        }
    }
}