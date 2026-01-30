// #[derive(Debug, Clone, Serialize, RegisterComponent)]

use gameplay::traits::field_override::FieldOverride;
use macro_component::facet;

#[facet]
pub struct ComponentViewPlayer {}
impl FieldOverride for ComponentViewPlayer {
    fn apply(&mut self, _field: &str, _val: &str) {}
}
// impl Default for ComponentViewPlayer {
//     fn default() -> Self {
//         Self { owner: None }
//     }
// }
