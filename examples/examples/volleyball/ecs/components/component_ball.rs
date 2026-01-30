// #[derive(Debug, Clone, Serialize, RegisterComponent)]

use gameplay::traits::field_override::FieldOverride;
use macro_component::facet;

#[facet]
pub struct ComponentBall {}
impl FieldOverride for ComponentBall {
    fn apply(&mut self, _field: &str, _val: &str) {}
}
