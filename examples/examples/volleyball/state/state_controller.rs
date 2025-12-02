use core::{collections::state_ownerships::StateOwnerships, system::system_game_state::IState};
use std::{collections::HashMap, hash::Hash};

use macro_state_serialize::global_state_serialize;

use crate::listeners::listener_start_encounter::Controller;

#[derive(PartialEq, Eq)]
#[global_state_serialize]
pub struct StateController {
    pub all_players: HashMap<i32, Controller>,
}
impl IState for StateController {
    fn id() -> i32 {
        90183012
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }
}
impl Hash for StateController {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut axis_keys: Vec<&i32> = self.all_players.keys().collect();
        axis_keys.sort();
        axis_keys.len().hash(state);
        for k in axis_keys {
            k.hash(state);
            self.all_players.get(k).unwrap().hash(state);
        }
    }
}
