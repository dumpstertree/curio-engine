use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::IState};
use std::{collections::HashMap, hash::Hash};

use macro_state_serialize::global_state_serialize;

use crate::state::state_deck::Deck;

#[derive(PartialEq, Eq)]
#[global_state_serialize]
pub struct StateDeckExploration {
    pub deck: HashMap<i32, Deck>,
}
impl IState for StateDeckExploration {
    fn id() -> i32 {
        99023234
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }
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
