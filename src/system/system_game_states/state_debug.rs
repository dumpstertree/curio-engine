use crate::system::system_game_state::IState;

#[derive(Clone)]
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
impl IState<StateDebug> for StateDebug {
    fn id() -> i32 {
        908234
    }
    fn default() -> StateDebug {
        StateDebug::new()
    }
}
