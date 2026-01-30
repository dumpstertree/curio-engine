use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::IState};
use std::{collections::HashMap, hash::Hash};

use record_serializable::record_serializable;

use crate::Assets;

#[derive(PartialEq, Eq)]
#[record_serializable]
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
        // for k in axis_keys {
        //     k.hash(state);
        //     self.all.get(k).unwrap().uid().hash(state);
        // }
    }
}
