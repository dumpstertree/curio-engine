use macro_component::global_component;
use system_component_default_gameplay::traits::field_override::FieldOverride;

#[global_component]

pub struct ComponentUIScoreState {}
impl FieldOverride for ComponentUIScoreState {
    fn apply(&mut self, _field: &str, _val: &str) {}
}
// impl Default for ComponentUIScoreState {
//     fn default() -> Self {
//         Self { owner: None }
//     }
// }
