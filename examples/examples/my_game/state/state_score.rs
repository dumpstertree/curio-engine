use core::{collections::state_ownerships::StateOwnerships, system::system_game_state::IState};

use macro_state_serialize::global_state_serialize;

#[derive(Hash, PartialEq, Eq)]
#[global_state_serialize]
pub struct StateScore {
    pub score: i32,
}
impl IState for StateScore {
    fn id() -> i32 {
        98274392
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }
}
