use core::{collections::state_ownerships::StateOwnerships, system::system_game_state::IState};
use std::{collections::HashMap, hash::Hash};

use macro_state_serialize::global_state_serialize;

#[derive(PartialEq, Eq)]
#[global_state_serialize]
pub struct StateEnergy {
    pub all_players: HashMap<i32, (i32, i32)>,
}
impl IState for StateEnergy {
    fn id() -> i32 {
        0001
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }
}
impl Hash for StateEnergy {
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
