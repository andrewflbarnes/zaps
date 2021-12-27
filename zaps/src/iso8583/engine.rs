extern crate hex;
use std::collections::HashMap;
use crate::{
    core::Parser,
    iso8583::{
        parse::{
            Iso8583ParseError,
            tokenise_next_bitmap,
            tokenise_next_bytes,
            tokenise_next_field,
        },
        spec::Spec,
    }
};

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

impl Parser<u16> for Iso8583Engine {
    type Err = Iso8583ParseError;

    fn tokenise(&self, payload: &[u8]) -> Result<std::collections::HashMap<u16, String>, Iso8583ParseError> {
        let mut pointer = 0;
        let mut tokens = HashMap::new();
        let mti = tokenise_next_bytes(payload, &mut pointer, 4)?
            .iter()
            .map(|b| *b as char)
            .collect::<String>();

        let mti_spec = self.spec.get_mti_spec(&mti)
            .ok_or(Iso8583ParseError::NoMtiDefinition)?;

        let mti_pri_bitmap = mti_spec.get(&0)
            .ok_or(Iso8583ParseError::NoTokenDefinition)?;

        let pri_bitmap = tokenise_next_bitmap(payload, &mut pointer, mti_pri_bitmap)?;

        for i in 1..=64 {
            let bitpos = i - 1;
            if 1 & (pri_bitmap >> (63 - bitpos)) == 1 {
                let field_value = tokenise_next_field(payload, &mut pointer, mti_spec, &i)?;

                tokens.insert(i, field_value);
                
            }
        }

        Ok(tokens)
    }
}