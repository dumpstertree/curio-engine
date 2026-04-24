use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::RecordCommon};
use std::{collections::HashMap, hash::Hash};

use record_serializable::record_serializable;

#[derive(PartialEq, Eq)]
#[record_serializable]
pub struct StateHealthExploration {
    pub all: HashMap<i32, (i32, i32)>,
}
impl RecordCommon for StateHealthExploration {
    fn id() -> i32 {
        990323234
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }
}
impl Hash for StateHealthExploration {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut keys: Vec<&i32> = self.all.keys().collect();
        keys.sort();
        keys.len().hash(state);
        for k in keys {
            k.hash(state);
            self.all.get(k).unwrap().hash(state);
        }
    }
}
