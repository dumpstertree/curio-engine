use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::RecordCommon};
use record_serializable::record_serializable;

use crate::state::state_teams::Teams;

#[derive(Hash, PartialEq, Eq)]
#[record_serializable]
pub struct StateTurn {
    pub active_instance_id: Teams,
}
impl RecordCommon for StateTurn {
    fn id() -> i32 {
        0005
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }
}
