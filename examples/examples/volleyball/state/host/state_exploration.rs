use curio_core::StateOwnerships;
use record_serializable::record_serializable;

use crate::exploration::exploration_path::Exploration;

#[derive(Hash, PartialEq, Eq)]
#[record_serializable(serializable,ownership = StateOwnerships::Host)]
pub struct StateExploration {
    pub exploration: Exploration,
    pub is_selecting_next: bool,
}
