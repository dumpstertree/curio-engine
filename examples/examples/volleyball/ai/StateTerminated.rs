use core::system::system_game_state::IState;
use std::hash::{Hash, Hasher};

#[derive(Clone)]
pub struct StateTerminated {
    pub is_terminated: bool,
    pub is_exhuasted: bool,
}

impl IState for StateTerminated {}
impl Hash for StateTerminated {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.is_terminated.hash(state);
        self.is_exhuasted.hash(state);
    }
}
