use gameplay::traits::field_override::FieldOverride;
use macro_component::facet;

#[facet]

pub struct ComponentUIScoreState {}
impl FieldOverride for ComponentUIScoreState {
    fn apply(&mut self, _field: &str, _val: &str) {}
}
// impl Default for ComponentUIScoreState {
//     fn default() -> Self {
//         Self { owner: None }
//     }
// }
