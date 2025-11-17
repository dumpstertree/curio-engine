use core::{collections::game_state::StateOwnerships, system::system_game_state::IState};

use macro_state_serialize::global_state_serialize;

#[derive(Hash, PartialEq, Eq)]
#[global_state_serialize]
pub struct StateTurn {
    pub active_instance_id: i32,
}
impl IState for StateTurn {
    fn id() -> i32 {
        0005
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }
}
