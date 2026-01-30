use gameplay::traits::field_override::FieldOverride;
use facet::facet;

// #[derive(Debug, Clone, Serialize, RegisterComponent)]
#[facet]

pub struct ComponentPlayer {
    pub player_id: i32,
}
impl ComponentPlayer {
    pub fn set_player_id(mut self, id: i32) -> Self {
        self.player_id = id;
        self
    }
}
impl FieldOverride for ComponentPlayer {
    fn apply(&mut self, _field: &str, _val: &str) {}
}
// impl Default for ComponentPlayer {
//     fn default() -> Self {
//         Self { owner: None, player_id: 0 }
//     }
// }
