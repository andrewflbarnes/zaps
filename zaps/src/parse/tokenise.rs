use std::collections::HashMap;

pub trait Tokeniser {
    type Err;

    fn tokenise(&self, payload: &[u8]) -> Result<HashMap<u16, String>, Self::Err>;
}