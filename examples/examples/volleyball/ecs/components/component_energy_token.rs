// #[derive(Debug, Clone, Serialize, RegisterComponent)]

use gameplay::traits::field_override::FieldOverride;
use macro_component::facet;

#[facet]
pub struct ComponentEnergyToken {
    pub index: i32,
}
impl ComponentEnergyToken {
    pub fn set_index(mut self, index: i32) -> Self {
        self.index = index;
        self
    }
}
impl FieldOverride for ComponentEnergyToken {
    fn apply(&mut self, _field: &str, _val: &str) {}
}
// impl Default for ComponentEnergyToken {
//     fn default() -> Self {
//         Self { owner: None, index: 0 }
//     }
// }
