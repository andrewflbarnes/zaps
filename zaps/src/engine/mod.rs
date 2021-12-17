extern crate hex;
use std::collections::HashMap;
use std::str;
use super::{
    DataType,
    Field,
    Spec,
    Tokeniser,
};


#[derive(Debug)]
pub enum Iso8583TokeniseError {
    EndOfData{
        field: u16,
        from: usize,
        count: usize,
        max: usize,
    },
    InvalidVarLength(u16, String),
    InvalidData(u16, Vec<u8>),
    NoTokenDefinition(u16),
    InvalidTokenDefinition(u16, Field),
}

pub struct Iso8583Engine {
    spec: Spec,
}

impl Iso8583Engine {
    pub fn new(spec: Spec) -> Self {
        Iso8583Engine{
            spec,
        }
    }
}

impl Tokeniser for Iso8583Engine {
    type Err = Iso8583TokeniseError;

    fn tokenise(&self, payload: &[u8]) -> Result<std::collections::HashMap<u16, String>, Iso8583TokeniseError> {
        let mut pointer = 0;
        let mut tokens = HashMap::new();
        let mti = tokenise_next(payload, &mut pointer, 4).iter()
            .map(|b| *b as char)
            .collect::<String>();

        println!("{}", mti);
        // TODO - resolve result properly
        let mti_spec = self.spec.get_mti_spec(&mti).unwrap();

        let mti_pri_bitmap = mti_spec.get(&0).unwrap();

        let pri_bitmap = as_bitmap(payload, &mut pointer, mti_pri_bitmap.size, &mti_pri_bitmap.data_type);

        // println!("{:b}", pri_bitmap);

        for i in 1..=64 {
            let bitpos = i - 1;
            if 1 & (pri_bitmap >> (63 - bitpos)) == 1 {
                let field = mti_spec.get(&i).unwrap();
                
                let field_size = match field.ftype.var_size_len() {
                    Some(field_size_len) => {
                        // TODO make this nicer, maybe use hex crate
                        let field_size_str = tokenise_next(payload, &mut pointer, field_size_len);
                        str::from_utf8(field_size_str)
                            .expect(&format!("Unable to convert {:?} to string", field_size_str))
                            .parse::<usize>()
                            .unwrap()
                    }
                    None => field.size,
                };

                let field_value_raw = tokenise_next(payload, &mut pointer, field_size);
                //TODO string bad....?
                let field_value = str::from_utf8(field_value_raw)
                    .expect(&format!("Unable to convert {:?} to string", field_value_raw))
                    .to_string();

                tokens.insert(i, field_value);
                
            }
        }

        // TODO validate field

        Ok(tokens)
    }
}

fn tokenise_next<'a>(payload: &'a[u8], pointer: &mut usize, size: usize) -> &'a [u8] {
    let result = &payload[*pointer..(*pointer + size)];

    *pointer += size;

    result
}

fn as_bitmap(payload: &[u8], pointer: &mut usize, size: usize, data_type: &DataType) -> u64 {
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

fn decode_ascii_bitmap(raw_bitmap: &[u8], size: usize) -> u64 {
    let mut decoded = [0u8; 8];
    hex::decode_to_slice(raw_bitmap, &mut decoded as &mut [u8])
        .expect(&format!("Unable to decode bitmap: {:?}", raw_bitmap));

    decode_bitmap(&decoded, size / 2)
}

fn decode_bitmap(raw_bitmap: &[u8], size: usize) -> u64 {
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