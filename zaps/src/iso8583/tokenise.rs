extern crate hex;
use std::collections::HashMap;
use std::convert::From;
use std::error;
use std::fmt;
use std::str;
use crate::{
    util::{
        byte_to_string,
        decode_ascii_bitmap,
        decode_bitmap,
        DecodeBitmapError,
    },
    core::{
        DataType,
        Field,
    }
};

#[derive(Debug, PartialEq)]
pub enum Iso8583TokeniseError {
    Overflow{
        from: usize,
        count: usize,
        max: usize,
    },
    InvalidVarLength(String),
    InvalidData(Vec<u8>),
    NoTokenDefinition,
    NoMtiDefinition,
    InvalidFieldDefinition,
    BadBitmap(DecodeBitmapError),
}

impl fmt::Display for Iso8583TokeniseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow{ from, count, max } => write!(f, "overflow reading from {} by {}, max {}", from, count, max),
            Self::InvalidVarLength(len) => write!(f, "invalid variable length: {}", len),
            Self::InvalidData(data) => write!(f, "invalid data read {:?}", data),
            Self::NoTokenDefinition => write!(f, "no token definition found"),
            Self::NoMtiDefinition => write!(f, "no MTI definition found"),
            Self::InvalidFieldDefinition => write!(f, "invalid field definition"),
            Self::BadBitmap(_bitmap_err) => write!(f, "invalid bitmap"),
        }
    }
}

impl error::Error for Iso8583TokeniseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::BadBitmap(bitmap_err) => Some(bitmap_err),
            _ => None
        }
    }
}

impl From<DecodeBitmapError> for Iso8583TokeniseError {
    fn from(err: DecodeBitmapError) -> Self {
        Self::BadBitmap(err)
    }
}



pub fn tokenise_next_bitmap(payload: &[u8], pointer: &mut usize, bitmap_defn: &Field) -> Result<u64, Iso8583TokeniseError> {
    let Field{ size, data_type, .. } = bitmap_defn;

    let raw_bitmap = tokenise_next_bytes(payload, pointer, *size)?;


    let bitmap = match data_type {
        DataType::Binary => {
            decode_bitmap(raw_bitmap, *size)?
        },
        DataType::Packed => {
            decode_ascii_bitmap(raw_bitmap, *size)?
        },
        _ => {
            return Err(Iso8583TokeniseError::InvalidFieldDefinition)
        }
    };

    Ok(bitmap)
}

pub fn tokenise_next_field(payload: &[u8], pointer: &mut usize, mti_spec: &HashMap<u16, Field>, field_num: &u16) -> Result<String, Iso8583TokeniseError> {
    let field = mti_spec.get(field_num)
        .ok_or_else(|| Iso8583TokeniseError::NoTokenDefinition)?;
    
    let field_size = get_field_length(payload, pointer, field)?;

    let field_value_raw = tokenise_next_bytes(payload, pointer, field_size)?;

    let field_value = str::from_utf8(field_value_raw)
        .map_err(|_err| Iso8583TokeniseError::InvalidData(Vec::from(field_value_raw)))?
        .to_string();

    Ok(field_value)
}

pub fn tokenise_next_bytes<'a>(payload: &'a[u8], pointer: &mut usize, size: usize) -> Result<&'a [u8], Iso8583TokeniseError> {
    if *pointer + size > payload.len() {
        return Err(Iso8583TokeniseError::Overflow{
            from: *pointer,
            count: size,
            max: payload.len(),
        });
    }
    let result = &payload[*pointer..(*pointer + size)];

    *pointer += size;

    Ok(result)
}

fn get_field_length(payload: &[u8], pointer: &mut usize, field: &Field) -> Result<usize, Iso8583TokeniseError> {
    let field_length = match field.ftype.var_size_len() {
        Some(field_size_len) => {
            let field_size_str = tokenise_next_bytes(payload, pointer, field_size_len)?;
            // TODO use a simpler, more efficient (maybe handwritten?) process
            str::from_utf8(field_size_str)
                .map_err(|_err| Iso8583TokeniseError::InvalidData(Vec::from(field_size_str)))?
                .parse::<usize>()
                .map_err(|_err| Iso8583TokeniseError::InvalidVarLength(byte_to_string(field_size_str)))?
        }
        None => field.size,
    };

    Ok(field_length)
}

#[cfg(test)]
mod test {
    use crate::core::{
        DataType,
        Field,
        FieldType,
    };
    use super::{
        get_field_length,
        tokenise_next_bytes,
        Iso8583TokeniseError,
    };

    mod get_field_length {
        use super::*;

        macro_rules! test_get_field_length {
            ($(
                $name:ident:
                $payload:literal $field_type:ident $field_size:literal
                =>
                len:$expected:literal pointer:$expected_p:literal;
            )*) => {
                $(
                    #[test]
                    fn $name() {
                        let field = Field::new(FieldType::$field_type, $field_size, DataType::Alpha);
                        let mut pointer = 0;
                        let result = get_field_length($payload.as_bytes(), &mut pointer, &field).unwrap();
                        assert_eq!($expected, result);
                        assert_eq!($expected_p, pointer);
                    }
                )*
            };
        }

        test_get_field_length! (
            lvar_field_len: "3ZZZZZZZZZ" LVar 0 => len:3 pointer:1;
            llvar_field_len: "32ZZZZZZZZ" LLVar 0 => len:32 pointer:2;
            lllvar_field_len: "321ZZZZZZZ" LLLVar 0 => len:321 pointer:3;
            fixed_field_len: "ZZZZZZZZZZ" Fixed 15 => len:15 pointer:0;
        );
    }

    mod tokenise_next_bytes {
        use super::*;

        const PAYLOAD: &'static str = "1234567890abcdeffedbca0987654321";
        const PAYLOAD_EMPTY: &'static str = "";

        macro_rules! test_tokenise_next_bytes {
            ($(
                $name:ident:
                $payload:ident $from:literal $size:literal
                =>
                $expected:literal;
            )*) => {
                $(
                    #[test]
                    fn $name() {
                        let mut pointer = $from;
                        let result = tokenise_next_bytes($payload.as_bytes(), &mut pointer, $size).unwrap();
                        assert_eq!($expected.as_bytes(), result);
                        assert_eq!($size + $from, pointer);
                    }
                )*
            };
        }

        test_tokenise_next_bytes!(
            no_length: PAYLOAD 0 0 => "";
            no_length_from: PAYLOAD 10 0 => "";
            no_length_empty: PAYLOAD_EMPTY 0 0 => "";
            some_length: PAYLOAD 0 10 => "1234567890";
            some_length_from: PAYLOAD 6 10 => "7890abcdef";
        );

        macro_rules! test_tokenise_next_bytes_overflow {
            ($(
                $name:ident:
                $payload:ident $from:literal $size:literal;
            )*) => {
                $(
                    #[test]
                    fn $name() {
                        let mut pointer = $from;
                        let result = tokenise_next_bytes($payload.as_bytes(), &mut pointer, $size);
                        assert_eq!(Err(Iso8583TokeniseError::Overflow{
                            from: $from,
                            count: $size,
                            max: $payload.len(),
                        }), result);
                    }
                )*
            };
        }

        test_tokenise_next_bytes_overflow!(
            overflow_by_many: PAYLOAD 0 100;
            overflow_by_one: PAYLOAD 0 33;
            overflow_end_by_many: PAYLOAD 32 100;
            overflow_end_by_one: PAYLOAD 32 1;
            overflow_empty_by_many: PAYLOAD_EMPTY 0 100;
            overflow_empty_by_one: PAYLOAD_EMPTY 0 1;
        );
    }
}