use std::collections::HashMap;

pub trait Unparser<K> {
    type Err;

    fn unparse(&self, fields: HashMap<K, String>, out:&mut [u8]) -> Result<(), Self::Err>;
}