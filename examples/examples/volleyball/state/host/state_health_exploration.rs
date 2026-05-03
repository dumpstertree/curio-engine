use std::{collections::HashMap, hash::Hash};

use curio_core::StateOwnerships;
use record_serializable::record_serializable;

#[derive(PartialEq, Eq)]
#[record_serializable(serializable,ownership = StateOwnerships::Host)]
pub struct StateHealthExploration {
    pub all: HashMap<i32, (i32, i32)>,
}
impl Hash for StateHealthExploration {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut keys: Vec<&i32> = self.all.keys().collect();
        keys.sort();
        keys.len().hash(state);
        for k in keys {
            k.hash(state);
            self.all.get(k).unwrap().hash(state);
        }
    }
}
