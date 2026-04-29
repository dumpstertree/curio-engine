use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::RecordCommon};

use record_serializable::record_serializable;
#[derive(Hash, PartialEq, Eq)]
#[record_serializable(serializable,ownership = StateOwnerships::Instance)]
pub struct StatePeerSelectedCards {
    pub index: i32,
}
