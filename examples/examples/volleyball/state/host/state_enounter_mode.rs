use curio_core::StateOwnerships;
use record_serializable::record_serializable;

use crate::listeners::listener_initialize_encounter::Encounter;

#[derive(Hash, PartialEq, Eq)]
#[record_serializable(serializable,ownership = StateOwnerships::Host)]
pub struct StateEncounter {
    pub encounter: Encounter,
}
