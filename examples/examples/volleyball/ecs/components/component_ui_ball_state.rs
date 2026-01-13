use macro_component::global_component;
use system_component_default_gameplay::traits::field_override::FieldOverride;

#[global_component]
pub struct ComponentUIBallState {}
impl FieldOverride for ComponentUIBallState {
    fn apply(&mut self, _field: &str, _val: &str) {}
}
// impl Default for ComponentUIBallState {
//     fn default() -> Self {
//         Self { owner: None }
//     }
// }
