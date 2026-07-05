use std::sync::OnceLock;

use crate::{FieldState, RecordCommon, SequentialRecordUIDs, RecordOverride};

static SYS_RECORD_ID: OnceLock<i32> = OnceLock::new();

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
    fn name(&self) -> String {
        String::from("Screen")
    }
    fn id() -> i32 {
        *SYS_RECORD_ID.get_or_init(|| SequentialRecordUIDs::of::<SysRecordScreen>())
    }
}
impl RecordOverride for SysRecordScreen {
    fn set_state(&mut self, _field: &str, _val: &str) {}
    fn get_state(&self) -> Vec<crate::FieldState> {
        vec![
            FieldState::new("width", self.width), //
            FieldState::new("height", self.height),
        ]
    }
}
