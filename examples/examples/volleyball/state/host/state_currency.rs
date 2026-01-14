use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::IState};
use macro_state_serialize::global_state_serialize;
use std::hash::Hash;

#[derive(Hash, PartialEq, Eq)]
#[global_state_serialize]
pub struct StateCurrency {
    pub currency: i32,
}
impl IState for StateCurrency {
    fn id() -> i32 {
        901812999
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }
}
