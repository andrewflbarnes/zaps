use std::collections::HashMap;

mod field;
mod builder;

pub use field::{
    DataType,
    Field,
    FieldParseError,
    FieldType,
};

#[derive(Debug)]
pub struct Spec {
    messages: HashMap<String, HashMap<u16, Field>>
}

impl Spec {
    pub fn new() -> Self {
        Spec{
            messages: HashMap::new()
        }
    }

    pub fn add_spec(&mut self, mti: String, fields: HashMap<u16, Field>) {
        self.messages.insert(mti, fields);
    }
}