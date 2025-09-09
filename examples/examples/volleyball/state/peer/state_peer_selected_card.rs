use core::{collections::game_state::StateOwnerships, system::system_game_state::IState};

use macro_state_serialize::global_state_serialize;

#[global_state_serialize]
pub struct StatePeerSelectedCards {
    pub index: i32,
}
impl IState for StatePeerSelectedCards {
    fn id() -> i32 {
        89375938
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Instance
    }
}
