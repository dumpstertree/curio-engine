use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::RecordCommon};
use std::{collections::HashMap, hash::Hash};

use record_serializable::record_serializable;

use crate::state::state_teams::Teams;

#[derive(PartialEq, Eq)]
#[record_serializable(ownership = StateOwnerships::Host)]
pub struct StateScore {
    pub all_scores: HashMap<Teams, i32>,
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
