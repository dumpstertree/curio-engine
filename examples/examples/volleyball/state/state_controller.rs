use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::RecordCommon};
use std::{collections::HashMap, hash::Hash};

use record_serializable::record_serializable;

use crate::listeners::listener_initialize_encounter::Controller;

#[derive(PartialEq, Eq)]
#[record_serializable]
pub struct StateController {
    pub all_players: HashMap<i32, Controller>,
}
impl RecordCommon for StateController {
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
