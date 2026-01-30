use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::IState};

use record_serializable::record_serializable;
use serde::{Deserialize, Serialize};

#[derive(Hash, PartialEq, Eq)]
#[record_serializable]
pub struct StatePeerInputMode {
    pub mode: InputModes,
}
impl IState for StatePeerInputMode {
    fn id() -> i32 {
        982734
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Instance
    }
}

#[derive(Default, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum InputModes {
    #[default]
    Move,
    Manuever,
}
