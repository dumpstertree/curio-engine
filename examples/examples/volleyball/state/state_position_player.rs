use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::RecordCommon};
use std::{collections::HashMap, hash::Hash};

use record_serializable::record_serializable;
#[derive(PartialEq, Eq)]
#[record_serializable(serializable,ownership = StateOwnerships::Host)]
pub struct StatePositionEntities {
    pub positions: HashMap<i32, (i32, i32)>,
}
impl Hash for StatePositionEntities {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut axis_keys: Vec<&i32> = self.positions.keys().collect();
        axis_keys.sort();
        axis_keys.len().hash(state);
        for k in axis_keys {
            k.hash(state);
            self.positions.get(k).unwrap().hash(state);
        }
    }
}
