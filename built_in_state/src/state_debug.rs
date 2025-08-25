use core::system::system_game_state::IState;

use macro_state::global_state;

#[global_state]
pub struct StateDebug {
    pub is_inspecting: bool,
    pub is_paused: bool,
}
impl StateDebug {
    pub fn new<'a>() -> StateDebug {
        StateDebug {
            is_inspecting: false,
            is_paused: false,
        }
    }
}
impl IState for StateDebug {
    fn id() -> i32 {
        908234
    }
}
