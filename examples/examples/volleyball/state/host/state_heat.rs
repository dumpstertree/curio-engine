use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::RecordCommon};
use record_serializable::record_serializable;
use std::{collections::HashMap, hash::Hash};

#[derive(PartialEq, Eq)]
#[record_serializable(ownership = StateOwnerships::Host)]
pub struct StateHeat {
    pub all_players: HashMap<i32, i32>,
}
impl Hash for StateHeat {
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
