use crate::system::system_game_state::RecordCommon;

#[derive(Default, Hash, PartialEq, Eq, Clone)]
pub struct SysRecordScreen {
    width: i32,
    height: i32,
}
impl SysRecordScreen {
    pub fn width(&self) -> i32 {
        self.width
    }
    pub fn height(&self) -> i32 {
        self.height
    }
    pub fn new<'a>(width: i32, height: i32) -> SysRecordScreen {
        SysRecordScreen { width, height }
    }
}
impl RecordCommon for SysRecordScreen {
    fn id() -> i32 {
        464
    }
}
