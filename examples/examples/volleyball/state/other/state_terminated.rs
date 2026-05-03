use curio_core::{RecordCommon, StateOwnerships, system::record_id::RecordId};
use std::{
    hash::{Hash, Hasher},
    sync::OnceLock,
};

static RECORD_ID: OnceLock<i32> = OnceLock::new();

#[derive(Clone, Default)]
pub struct StateTerminated {
    pub is_terminated: bool,
    pub is_exhuasted: bool,
}

impl RecordCommon for StateTerminated {
    fn ownership() -> StateOwnerships {
        StateOwnerships::Instance
    }

    fn id() -> i32 {
        *RECORD_ID.get_or_init(|| RecordId::of::<StateTerminated>())
    }
}

impl Hash for StateTerminated {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.is_terminated.hash(state);
        self.is_exhuasted.hash(state);
    }
}
