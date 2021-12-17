use crate::core::DataType;

pub fn as_bitmap(payload: &[u8], pointer: &mut usize, size: usize, data_type: &DataType) -> u64 {
    let raw_bitmap = &payload[*pointer..(*pointer + size)];

    *pointer += size;

    match data_type {
        DataType::Binary => {
            decode_bitmap(raw_bitmap, size)
        },
        DataType::Packed => {
            decode_ascii_bitmap(raw_bitmap, size)
        },
        _ => {
            todo!();
        }
    }
}

pub fn decode_ascii_bitmap(raw_bitmap: &[u8], size: usize) -> u64 {
    let mut decoded = vec![0u8; size / 2];
    hex::decode_to_slice(raw_bitmap, &mut decoded as &mut [u8])
        .expect(&format!("Unable to decode bitmap: {:?}", raw_bitmap));

    decode_bitmap(&decoded, size / 2)
}

pub fn decode_bitmap(raw_bitmap: &[u8], size: usize) -> u64 {
    let mut bitmap = 0u64;
    for i in 0..size {
        let byte = raw_bitmap[i];
        for j in 0..8 {
            let bit = 7 - j;
            if 1 & (byte >> bit) == 1 {
                let shift = 8 * (7 - i) + 7 - j;
                bitmap += 1 << shift;
            }
        }
    }
    bitmap
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
        let bitmap = decode_ascii_bitmap(&raw_bitmap, raw_bitmap.len());

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
        let bitmap = decode_bitmap(&raw_bitmap, raw_bitmap.len());

        for i in 0..64 {
            let bitpos = 63 - i;
            match i {
                0 | 8 | 41 | 63 => assert!(1 & (bitmap >> bitpos) == 1),
                _ => assert!(1 & (bitmap >> bitpos) == 0),
            }
        }

    }
}