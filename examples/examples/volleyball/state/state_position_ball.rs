use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::RecordCommon};

use record_serializable::record_serializable;
#[derive(Hash, PartialEq, Eq)]
#[record_serializable(ownership = StateOwnerships::Host)]
pub struct StatePositionBall {
    pub row: i32,
    pub column: i32,
}
