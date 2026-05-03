use std::{collections::HashMap, hash::Hash};

use curio_core::StateOwnerships;
use record_serializable::record_serializable;

use crate::listeners::listener_initialize_encounter::Controller;

#[derive(PartialEq, Eq)]
#[record_serializable(serializable,ownership = StateOwnerships::Host)]
pub struct StateController {
    pub all_players: HashMap<i32, Controller>,
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
