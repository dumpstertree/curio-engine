use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::IState};

use macro_state_serialize::global_state_serialize;
use serde::{Deserialize, Serialize};

#[derive(Hash, PartialEq, Eq)]
#[global_state_serialize]
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
