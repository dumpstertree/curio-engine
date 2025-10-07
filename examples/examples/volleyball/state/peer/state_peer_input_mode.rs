use core::{collections::game_state::StateOwnerships, system::system_game_state::IState};

use macro_state_serialize::global_state_serialize;
use serde::{Deserialize, Serialize};

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

#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InputModes {
    #[default]
    Move,
    Manuever,
}
