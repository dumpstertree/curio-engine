use facet::facet;
use gameplay::traits::field_override::FieldOverride;

#[facet]

pub struct ComponentUITurnState {}
impl FieldOverride for ComponentUITurnState {
    fn apply(&mut self, _field: &str, _val: &str) {}
}
// impl Default for ComponentUITurnState {
//     fn default() -> Self {
//         Self { owner: None }
//     }
// }
