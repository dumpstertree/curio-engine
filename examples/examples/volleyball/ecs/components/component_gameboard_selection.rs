use macro_component::global_component;
use system_component_default_gameplay::traits::field_override::FieldOverride;

#[global_component]
pub struct ComponentGameBoardSelection {}
impl FieldOverride for ComponentGameBoardSelection {
    fn apply(&mut self, _field: &str, _val: &str) {}
}
