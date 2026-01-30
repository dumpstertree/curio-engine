use facet::facet;
use gameplay::traits::field_override::FieldOverride;

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
