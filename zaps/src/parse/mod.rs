use std::collections::HashMap;

pub trait Parser<K> {
    type Err;

    fn parse(&self, payload: &[u8]) -> Result<HashMap<K, String>, Self::Err>;
}