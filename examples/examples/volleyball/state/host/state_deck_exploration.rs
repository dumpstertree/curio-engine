use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::RecordCommon};
use std::{collections::HashMap, hash::Hash};

use record_serializable::record_serializable;

use crate::state::state_deck::Deck;

#[derive(PartialEq, Eq)]
#[record_serializable(ownership = StateOwnerships::Host)]
pub struct StateDeckExploration {
    pub deck: HashMap<i32, Deck>,
}

impl Hash for StateDeckExploration {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut keys: Vec<&i32> = self.deck.keys().collect();
        keys.sort();
        keys.len().hash(state);
        for k in keys {
            k.hash(state);
            self.deck.get(k).unwrap().hash(state);
        }
    }
}
