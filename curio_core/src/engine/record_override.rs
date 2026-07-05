use crate::FieldState;

/// A helper trait is used to give a record the ability to be restored from data
pub trait RecordOverride {
    fn set_state(&mut self, field: &str, val: &str);
    fn get_state(&self) -> Vec<FieldState>;
}
