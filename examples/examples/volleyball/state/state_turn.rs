use curio_core::StateOwnerships;
use record_serializable::record_serializable;

use crate::state::state_teams::Teams;

#[derive(Hash, PartialEq, Eq)]
#[record_serializable(serializable,ownership = StateOwnerships::Host)]
pub struct StateTurn {
    pub active_instance_id: Teams,
}
