use macro_component::facet;
use system_component_default_gameplay::traits::field_override::FieldOverride;

#[facet]
pub struct ComponentGameBoardSelection {}
impl FieldOverride for ComponentGameBoardSelection {
    fn apply(&mut self, _field: &str, _val: &str) {}
}
