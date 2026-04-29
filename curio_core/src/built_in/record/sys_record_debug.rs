use crate::system::system_game_state::RecordCommon;

#[derive(Default, Hash, Clone)]
pub struct SysRecordDebug {
    pub is_inspecting: bool,
    pub is_paused: bool,
}
impl SysRecordDebug {
    pub fn new<'a>() -> SysRecordDebug {
        SysRecordDebug { is_inspecting: false, is_paused: false }
    }
}
impl RecordCommon for SysRecordDebug {
    fn id() -> i32 {
        104
    }
}
