use core::{collections::state_ownerships::StateOwnerships, system::system_game_state::IState};
use macro_state_serialize::global_state_serialize;
use std::{collections::HashMap, hash::Hash, sync::Arc};

use crate::cards::{card_dependencies::filled_card_response::FilledCardResponse, card_instance::CardInstance};

#[derive(PartialEq, Eq)]
#[global_state_serialize]
pub struct StatePlayHistory {
    pub history: Vec<(i32, Arc<CardInstance>, FilledCardResponse)>,
}
impl IState for StatePlayHistory {
    fn id() -> i32 {
        911830129
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }
}
impl Hash for StatePlayHistory {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // let mut axis_keys: Vec<&i32> = self.history.keys().collect();
        // axis_keys.sort();
        // axis_keys.len().hash(state);
        // for k in axis_keys {
        //     k.hash(state);
        //     self.history.get(k).unwrap().hash(state);
        // }
    }
}
