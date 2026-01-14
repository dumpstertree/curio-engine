use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::IState};
use std::{collections::HashMap, hash::Hash};

use macro_state_serialize::global_state_serialize;

#[derive(PartialEq, Eq)]
#[global_state_serialize]
pub struct StateHealthExploration {
    pub all: HashMap<i32, (i32, i32)>,
}
impl IState for StateHealthExploration {
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
