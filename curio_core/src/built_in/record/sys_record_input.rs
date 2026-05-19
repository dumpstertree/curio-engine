use std::sync::OnceLock;

use crate::{input::input_mapped::InputMapped, system::record_id::RecordId, InputRaw, RecordCommon};

static SYS_RECORD_ID: OnceLock<i32> = OnceLock::new();

#[derive(Default, Hash, Clone, PartialEq, Eq)]
pub struct SysRecordInput {
    pub mapped: Vec<InputMapped>,
    pub raw: InputRaw,
}

impl SysRecordInput {
    pub fn default() -> SysRecordInput {
        SysRecordInput { mapped: Vec::new(), raw: InputRaw::new() }
    }
}
impl RecordCommon for SysRecordInput {
    fn id() -> i32 {
        *SYS_RECORD_ID.get_or_init(|| RecordId::of::<InputRaw>())
    }
}
