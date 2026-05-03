use std::sync::OnceLock;

use crate::{system::record_id::RecordId, RecordCommon};

static SYS_RECORD_ID: OnceLock<i32> = OnceLock::new();

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
        *SYS_RECORD_ID.get_or_init(|| RecordId::of::<SysRecordDebug>())
    }
}
