use core::{collections::draw_call::DrawCall, system::system_game_state::IState};

use macro_state::global_state;

#[derive(Hash)]
#[global_state]
pub struct DrawCallsState {
    pub draw_calls: Vec<DrawCall>,
}
impl DrawCallsState {
    pub fn new<'a>() -> DrawCallsState {
        DrawCallsState { draw_calls: Vec::new() }
    }
}
impl IState for DrawCallsState {
    fn id() -> i32 {
        12345
    }
}
