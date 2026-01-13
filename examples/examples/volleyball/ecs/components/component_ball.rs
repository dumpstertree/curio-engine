// #[derive(Debug, Clone, Serialize, RegisterComponent)]

use macro_component::facet;
use system_component_default_gameplay::traits::field_override::FieldOverride;

#[facet]
pub struct ComponentBall {}
impl FieldOverride for ComponentBall {
    fn apply(&mut self, _field: &str, _val: &str) {}
}
