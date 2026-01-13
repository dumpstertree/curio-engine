use macro_component::global_component;
use system_component_default_gameplay::traits::field_override::FieldOverride;

#[global_component]

pub struct ComponentUITurnState {}
impl FieldOverride for ComponentUITurnState {
    fn apply(&mut self, _field: &str, _val: &str) {}
}
// impl Default for ComponentUITurnState {
//     fn default() -> Self {
//         Self { owner: None }
//     }
// }
