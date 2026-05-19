use curio_core::FieldState;

pub trait FieldOverride {
    fn apply(&mut self, field: &str, val: &str);
    fn get_state(&self) -> Vec<FieldState>;
}
