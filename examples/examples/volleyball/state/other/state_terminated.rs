use curio_core::system::system_game_state::RecordCommon;
use std::hash::{Hash, Hasher};

#[derive(Clone)]
pub struct StateTerminated {
    pub is_terminated: bool,
    pub is_exhuasted: bool,
}

impl RecordCommon for StateTerminated {}
impl Hash for StateTerminated {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.is_terminated.hash(state);
        self.is_exhuasted.hash(state);
    }
}
