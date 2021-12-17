extern crate hex;
use std::collections::HashMap;
use std::str;
use crate::{
    util::{
        as_bitmap,
        byte_to_string,
    },
    core::{
        Field,
        Spec,
        Tokeniser,
    },
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
    NoMtiDefinition(String),
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
        let mti = tokenise_next_bytes(payload, &mut pointer, 4).iter()
            .map(|b| *b as char)
            .collect::<String>();

        let mti_spec = self.spec.get_mti_spec(&mti)
            .ok_or(Iso8583TokeniseError::NoMtiDefinition(mti))?;

        let mti_pri_bitmap = mti_spec.get(&0)
            .ok_or(Iso8583TokeniseError::NoTokenDefinition(0))?;

        let pri_bitmap = as_bitmap(payload, &mut pointer, mti_pri_bitmap.size, &mti_pri_bitmap.data_type);

        for i in 1..=64 {
            let bitpos = i - 1;
            if 1 & (pri_bitmap >> (63 - bitpos)) == 1 {
                let field_value = tokenise_next(payload, &mut pointer, mti_spec, &i)?;

                tokens.insert(i, field_value);
                
            }
        }

        Ok(tokens)
    }
}

fn tokenise_next(payload: &[u8], pointer: &mut usize, mti_spec: &HashMap<u16, Field>, field_num: &u16) -> Result<String, Iso8583TokeniseError> {
    let field = mti_spec.get(field_num)
        .ok_or_else(|| Iso8583TokeniseError::NoTokenDefinition(*field_num))?;
    
    let field_size = match field.ftype.var_size_len() {
        Some(field_size_len) => {
            let field_size_str = tokenise_next_bytes(payload, pointer, field_size_len);
            str::from_utf8(field_size_str)
                .map_err(|_err| Iso8583TokeniseError::InvalidData(*field_num, Vec::from(field_size_str)))?
                .parse::<usize>()
                .map_err(|_err| Iso8583TokeniseError::InvalidVarLength(*field_num, byte_to_string(field_size_str)))?
        }
        None => field.size,
    };

    let field_value_raw = tokenise_next_bytes(payload, pointer, field_size);
    let field_value = str::from_utf8(field_value_raw)
        .map_err(|_err| Iso8583TokeniseError::InvalidData(*field_num, Vec::from(field_value_raw)))?
        .to_string();

    Ok(field_value)

}

fn tokenise_next_bytes<'a>(payload: &'a[u8], pointer: &mut usize, size: usize) -> &'a [u8] {
    let result = &payload[*pointer..(*pointer + size)];

    *pointer += size;

    result
}