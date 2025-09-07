use core::{collections::game_state::StateOwnerships, system::system_game_state::IState};
use std::collections::HashMap;

use macro_state_serialize::global_state_serialize;

#[global_state_serialize]
pub struct StateEnergy {
    pub all_players: HashMap<i32, (i32, i32)>,
}
impl IState for StateEnergy {
    fn id() -> i32 {
        0001
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }
}
