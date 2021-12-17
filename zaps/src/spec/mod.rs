use std::{collections::HashMap};

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
    message_specs: HashMap<String, HashMap<u16, Field>>
}

impl Spec {
    pub fn new() -> Self {
        Spec{
            message_specs: HashMap::new()
        }
    }

    pub fn add_mti_spec(&mut self, mti: String, fields: HashMap<u16, Field>) {
        self.message_specs.insert(mti, fields);
    }

    pub fn get_mti_specs(&self) -> &HashMap<String, HashMap<u16, Field>> {
        &self.message_specs
    }

    pub fn has_mti_spec(&self, mti: &String) -> bool {
        self.message_specs.contains_key(mti)
    }

    pub fn get_mti_spec(&self, mti: &String) -> Option<&HashMap<u16, Field>> {
        self.message_specs.get(mti)
    }
}