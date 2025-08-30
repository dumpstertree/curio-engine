use core::{collections::game_state::StateOwnerships, system::system_game_state::IState};

use macro_state_serialize::global_state_serialize;

#[global_state_serialize]
pub struct StateEnergy {
    pub cur_energy: i32,
    pub max_energy: i32,
}
impl IState for StateEnergy {
    fn id() -> i32 {
        0001
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }
}
