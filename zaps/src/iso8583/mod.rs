extern crate hex;
use std::collections::HashMap;
use std::str;
use crate::{
    util::as_bitmap,
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
        let mti = tokenise_next(payload, &mut pointer, 4).iter()
            .map(|b| *b as char)
            .collect::<String>();

        // println!("{}", mti);
        let mti_spec = self.spec.get_mti_spec(&mti)
            .ok_or_else(|| Iso8583TokeniseError::NoMtiDefinition(mti.into()))?;

        let mti_pri_bitmap = mti_spec.get(&0)
            .ok_or_else(|| Iso8583TokeniseError::NoTokenDefinition(0))?;

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