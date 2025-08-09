use crate::{system::system_game_state::IState, Collections::DrawCall::DrawCall};

#[derive(Clone)]
pub struct DrawCallsState {
    pub draw_calls: Vec<DrawCall>,
}
impl DrawCallsState {
    pub fn new<'a>() -> DrawCallsState {
        DrawCallsState { draw_calls: Vec::new() }
    }
}
impl IState<DrawCallsState> for DrawCallsState {
    fn id() -> i32 {
        12345
    }
    fn default() -> DrawCallsState {
        DrawCallsState::new()
    }
}
