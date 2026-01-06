pub trait FieldDeserialize {
    fn override_field(&mut self, field: &str, val: &str);
}
