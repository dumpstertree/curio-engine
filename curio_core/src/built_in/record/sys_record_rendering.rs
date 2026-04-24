use crate::{system::system_game_state::RecordCommon, DrawCall};

#[derive(Default, Hash, Clone)]
pub struct SysRecordRendering {
    pub draw_calls: Vec<DrawCall>,
}
impl SysRecordRendering {
    pub fn new<'a>() -> SysRecordRendering {
        SysRecordRendering { draw_calls: Vec::new() }
    }
}
impl RecordCommon for SysRecordRendering {
    fn id() -> i32 {
        12345
    }
}
