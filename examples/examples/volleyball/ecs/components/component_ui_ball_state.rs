use gameplay::traits::field_override::FieldOverride;
use macro_component::facet;

#[facet]
pub struct ComponentUIBallState {}
impl FieldOverride for ComponentUIBallState {
    fn apply(&mut self, _field: &str, _val: &str) {}
}
// impl Default for ComponentUIBallState {
//     fn default() -> Self {
//         Self { owner: None }
//     }
// }
