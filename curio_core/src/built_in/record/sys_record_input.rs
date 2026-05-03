use std::sync::OnceLock;

use crate::{
    input::{input_snapshot_mapped::PlayerInputSnapshot, input_snapshot_raw::RawInputSnapshot},
    system::record_id::RecordId,
    RecordCommon,
};

static SYS_RECORD_ID: OnceLock<i32> = OnceLock::new();

#[derive(Default, Hash, Clone, PartialEq, Eq)]
pub struct SysRecordInput {
    pub mapped: Vec<PlayerInputSnapshot>,
    pub raw: RawInputSnapshot,
}

impl SysRecordInput {
    pub fn default() -> SysRecordInput {
        SysRecordInput { mapped: Vec::new(), raw: RawInputSnapshot::new() }
    }
}
impl RecordCommon for SysRecordInput {
    fn id() -> i32 {
        *SYS_RECORD_ID.get_or_init(|| RecordId::of::<RawInputSnapshot>())
    }
}
