use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::RecordCommon};

use record_serializable::record_serializable;

use crate::listeners::listener_initialize_encounter::Encounter;

#[derive(Hash, PartialEq, Eq)]
#[record_serializable]
pub struct StateEncounter {
    pub encounter: Encounter,
}
impl RecordCommon for StateEncounter {
    fn id() -> i32 {
        990249234
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Instance
    }

    fn default_box() -> Box<dyn RecordCommon>
    where
        Self: Sized + Default + 'static,
    {
        Box::new(Self::default())
    }
}
