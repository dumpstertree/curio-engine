use core::{collections::game_state::StateOwnerships, system::system_game_state::IState};
use std::collections::HashMap;

use macro_state_serialize::global_state_serialize;

#[global_state_serialize]
pub struct StatePositionPlayer {
    pub positions: HashMap<i32, (i32, i32)>,
}
impl IState for StatePositionPlayer {
    fn id() -> i32 {
        0004
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }
}
