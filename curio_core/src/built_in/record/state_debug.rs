use crate::system::system_game_state::IState;

#[derive(Default, Hash, Clone)]
pub struct StateDebug {
    pub is_inspecting: bool,
    pub is_paused: bool,
}
impl StateDebug {
    pub fn new<'a>() -> StateDebug {
        StateDebug { is_inspecting: false, is_paused: false }
    }
}
impl IState for StateDebug {
    fn id() -> i32 {
        908234
    }
}
