use core::{collections::game_state::StateOwnerships, system::system_game_state::IState};
use std::{collections::HashMap, hash::Hash};

use macro_state_serialize::global_state_serialize;

use crate::state::state_teams::Teams;

#[derive(PartialEq, Eq)]
#[global_state_serialize]
pub struct StateScore {
    pub all_scores: HashMap<Teams, i32>,
}
impl IState for StateScore {
    fn id() -> i32 {
        90809
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }
}
impl Hash for StateScore {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut axis_keys: Vec<&Teams> = self.all_scores.keys().collect();
        axis_keys.sort();
        axis_keys.len().hash(state);
        for k in axis_keys {
            k.hash(state);
            self.all_scores.get(k).unwrap().hash(state);
        }
    }
}
