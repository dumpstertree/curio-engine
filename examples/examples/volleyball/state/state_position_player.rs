use core::{collections::state_ownerships::StateOwnerships, system::system_game_state::IState};
use std::{collections::HashMap, hash::Hash};

use macro_state_serialize::global_state_serialize;
#[derive(PartialEq, Eq)]
#[global_state_serialize]
pub struct StatePositionPlayer {
    pub positions: HashMap<i32, (i32, i32)>,
}
impl Hash for StatePositionPlayer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut axis_keys: Vec<&i32> = self.positions.keys().collect();
        axis_keys.sort();
        axis_keys.len().hash(state);
        for k in axis_keys {
            k.hash(state);
            self.positions.get(k).unwrap().hash(state);
        }
    }
}

impl IState for StatePositionPlayer {
    fn id() -> i32 {
        0004
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }
}
