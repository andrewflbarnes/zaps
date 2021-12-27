extern crate hex;
use std::collections::HashMap;
use crate::{
    core::Parser,
    iso8583::{
        engine::Iso8583Engine,
        parse::{
            Iso8583ParseError,
            tokenise_next_bitmap,
            tokenise_next_bytes,
            tokenise_next_field,
        },
    }
};

impl Parser<u16> for Iso8583Engine {
    type Err = Iso8583ParseError;

    fn parse(&self, payload: &[u8]) -> Result<std::collections::HashMap<u16, String>, Iso8583ParseError> {
        let mut pointer = 0;
        let mut tokens = HashMap::new();
        let mti = tokenise_next_bytes(payload, &mut pointer, 4)?
            .iter()
            .map(|b| *b as char)
            .collect::<String>();

        tokens.insert(0xffff, mti.clone());

        let mti_spec = self.spec.get_mti_spec(&mti)
            .ok_or(Iso8583ParseError::NoMtiDefinition)?;

        let mti_pri_bitmap = mti_spec.get(&0)
            .ok_or(Iso8583ParseError::NoTokenDefinition)?;

        let pri_bitmap = tokenise_next_bitmap(payload, &mut pointer, mti_pri_bitmap)?;

        let mut bitmap_str = format!("{:0b}", pri_bitmap);
        bitmap_str.truncate(mti_pri_bitmap.raw_size);
        tokens.insert(0, bitmap_str);

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