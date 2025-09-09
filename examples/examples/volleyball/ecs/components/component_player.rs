// #[derive(Debug, Clone, Serialize, RegisterComponent)]
pub struct ComponentPlayer {
    pub player_id: i32,
}
impl ComponentPlayer {
    pub fn default() -> ComponentPlayer {
        ComponentPlayer { player_id: 0 }
    }
    pub fn set_player_id(mut self, id: i32) -> Self {
        self.player_id = id;
        self
    }
}
