use crate::{
    input::{input_snapshot_mapped::PlayerInputSnapshot, input_snapshot_raw::RawInputSnapshot},
    system::system_game_state::RecordCommon,
};

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
        290873492
    }
}
