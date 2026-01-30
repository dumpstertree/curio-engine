use crate::{collections::draw_call::DrawCall, system::system_game_state::IState};

#[derive(Default, Hash, Clone)]
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
