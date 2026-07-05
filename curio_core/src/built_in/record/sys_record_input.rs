use std::sync::OnceLock;

use crate::{input::input_mapped::InputMapped, FieldState, InputRaw, RecordCommon, RecordOverride, SequentialRecordUIDs};

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
    fn name(&self) -> String {
        String::from("Input")
    }
    fn id() -> i32 {
        *SYS_RECORD_ID.get_or_init(|| SequentialRecordUIDs::of::<SysRecordInput>())
    }
}
impl RecordOverride for SysRecordInput {
    fn set_state(&mut self, _field: &str, _val: &str) {}
    fn get_state(&self) -> Vec<crate::FieldState> {
        vec![
            FieldState::new("mapped", &self.mapped), //
            FieldState::new("raw", &self.raw),
        ]
    }
}
