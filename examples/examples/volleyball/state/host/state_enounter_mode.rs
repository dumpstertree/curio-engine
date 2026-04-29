use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::RecordCommon};

use record_serializable::record_serializable;

use crate::listeners::listener_initialize_encounter::Encounter;

#[derive(Hash, PartialEq, Eq)]
#[record_serializable(ownership = StateOwnerships::Host)]
pub struct StateEncounter {
    pub encounter: Encounter,
}
