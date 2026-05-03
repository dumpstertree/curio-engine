use curio_core::StateOwnerships;
use record_serializable::record_serializable;
use std::{hash::Hash, sync::Arc};

use crate::cards::{card_dependencies::filled_card_response::FilledCardResponse, card_instance::CardInstance};

#[derive(PartialEq, Eq)]
#[record_serializable(serializable,ownership = StateOwnerships::Host)]
pub struct StatePlayHistory {
    pub history: Vec<(i32, Arc<CardInstance>, FilledCardResponse)>,
}
impl Hash for StatePlayHistory {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {
        // let mut axis_keys: Vec<&i32> = self.history.keys().collect();
        // axis_keys.sort();
        // axis_keys.len().hash(state);
        // for k in axis_keys {
        //     k.hash(state);
        //     self.history.get(k).unwrap().hash(state);
        // }
    }
}
