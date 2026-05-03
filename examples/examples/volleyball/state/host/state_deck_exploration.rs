use std::{collections::HashMap, hash::Hash};

use curio_core::StateOwnerships;
use record_serializable::record_serializable;

use crate::state::state_deck::Deck;

#[derive(PartialEq, Eq)]
#[record_serializable(serializable,ownership = StateOwnerships::Host)]
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
