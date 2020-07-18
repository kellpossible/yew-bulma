use form_validation::ValidationErrors;
use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

pub trait FieldKey: Clone + PartialEq + Display + Hash + Eq + Debug {}

impl FieldKey for &str {}

pub trait FieldLink<Key: Clone>: Debug {
    fn field_key(&self) -> &Key;
    fn send_message(&self, msg: FieldMsg);
}

pub trait FormField<Key> {
    fn validation_errors(&self) -> &ValidationErrors<Key>;
    fn field_key(&self) -> &Key;
}

#[derive(Copy, Clone)]
pub enum FieldMsg {
    Validate,
}
