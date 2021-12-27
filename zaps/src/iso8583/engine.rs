use crate::{
    iso8583::{
        spec::Spec,
    }
};

pub struct Iso8583Engine {
    pub(super) spec: Spec,
}

impl Iso8583Engine {
    pub fn new(spec: Spec) -> Self {
        Iso8583Engine{
            spec,
        }
    }
}