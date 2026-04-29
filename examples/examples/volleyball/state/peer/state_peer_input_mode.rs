use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::RecordCommon};

use record_serializable::record_serializable;
use serde::{Deserialize, Serialize};

#[derive(Hash, PartialEq, Eq)]
#[record_serializable(ownership = StateOwnerships::Instance)]
pub struct StatePeerInputMode {
    pub mode: InputModes,
}

#[derive(Default, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum InputModes {
    #[default]
    Move,
    Manuever,
}
