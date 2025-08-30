use core::{collections::game_state::StateOwnerships, system::system_game_state::IState};

use macro_state_serialize::global_state_serialize;

#[global_state_serialize]
pub struct StatePositionPlayer {
    pub row: i32,
    pub collun: i32,
}
impl IState for StatePositionPlayer {
    fn id() -> i32 {
        0004
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }
}
