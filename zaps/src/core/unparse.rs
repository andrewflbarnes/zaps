use std::collections::HashMap;

pub trait Unparser<K> {
    type Err;

    fn unparse(&self, payload: HashMap<K, String>, out:&mut [u8]) -> Result<(), Self::Err>;
}