use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::RecordCommon};
use std::hash::{Hash, Hasher};

#[derive(Clone, Default)]
pub struct StateTerminated {
    pub is_terminated: bool,
    pub is_exhuasted: bool,
}

impl RecordCommon for StateTerminated {
    fn ownership() -> StateOwnerships {
        StateOwnerships::Instance
    }

    fn id() -> i32
    where
        Self: Sized + 'static,
    {
        println!("TODO FIX THIS");
        1000
    }
}
impl Hash for StateTerminated {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.is_terminated.hash(state);
        self.is_exhuasted.hash(state);
    }
}
