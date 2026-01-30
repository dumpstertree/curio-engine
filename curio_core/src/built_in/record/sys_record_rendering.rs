use crate::{collections::draw_call::DrawCall, system::system_game_state::IState};

#[derive(Default, Hash, Clone)]
pub struct SysRecordRendering {
    pub draw_calls: Vec<DrawCall>,
}
impl SysRecordRendering {
    pub fn new<'a>() -> SysRecordRendering {
        SysRecordRendering { draw_calls: Vec::new() }
    }
}
impl IState for SysRecordRendering {
    fn id() -> i32 {
        12345
    }
}
