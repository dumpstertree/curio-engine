pub trait FieldOverride {
    fn apply(&mut self, field: &str, val: &str);
}
