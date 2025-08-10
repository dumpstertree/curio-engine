use crate::{system::system_game_state::IState, collections::gizmo::Gizmo};

#[derive(Clone)]
pub struct GizmosState {
    pub draw_calls: Vec<Gizmo>,
}
impl GizmosState {
    pub fn new<'a>() -> GizmosState {
        GizmosState { draw_calls: Vec::new() }
    }
}
impl IState<GizmosState> for GizmosState {
    fn id() -> i32 {
        9827234
    }
    fn default() -> GizmosState {
        GizmosState::new()
    }
}
