use std::sync::OnceLock;

use crate::{
    built_in::record::sys_record_camera::SysRecordCamera,
    system::{record_id::RecordId, system_game_state::RecordCommon},
    DrawCall,
};

static SYS_RECORD_ID: OnceLock<i32> = OnceLock::new();

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
        *SYS_RECORD_ID.get_or_init(|| RecordId::of::<SysRecordRendering>())
    }
}
