use core::{collections::state_ownerships::StateOwnerships, system::system_game_state::IState};
use std::{collections::HashMap, hash::Hash};

use macro_state_serialize::global_state_serialize;

use crate::Assets;

#[derive(PartialEq, Eq)]
#[global_state_serialize]
pub struct StateVisualEntity {
    pub all: HashMap<i32, Assets>,
}
impl IState for StateVisualEntity {
    fn id() -> i32 {
        90118301
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }
}
impl Hash for StateVisualEntity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut axis_keys: Vec<&i32> = self.all.keys().collect();
        axis_keys.sort();
        axis_keys.len().hash(state);
        for k in axis_keys {
            k.hash(state);
            self.all.get(k).unwrap().uid().hash(state);
        }
    }
}
