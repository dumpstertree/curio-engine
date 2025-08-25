use core::{collections::gizmo::Gizmo, system::system_game_state::IState};

use macro_state::global_state;

#[global_state]
pub struct GizmosState {
    pub draw_calls: Vec<Gizmo>,
}
impl GizmosState {
    pub fn new<'a>() -> GizmosState {
        GizmosState { draw_calls: Vec::new() }
    }
}
impl IState for GizmosState {
    fn id() -> i32 {
        9827234
    }
}
