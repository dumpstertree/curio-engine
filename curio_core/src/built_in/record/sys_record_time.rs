use std::{hash::Hash, sync::OnceLock};

static SYS_RECORD_ID: OnceLock<i32> = OnceLock::new();

use crate::{
    extensions::{extensions_f32::ExtensionsF32, extensions_f64::ExtensionsF64},
    system::{record_common::RecordOverride, record_id::RecordId},
    FieldState, RecordCommon,
};

#[derive(Default, PartialEq, Clone)]
pub struct SysRecordTime {
    pub target_frame_rate: f32,
    pub scaled_time: f64,
    pub unscaled_time: f64,
    pub frame_num: i64,
    pub unscaled_delta_time: f32,
    pub scaled_delta_time: f32,
    pub average_fps: i32,
}
impl SysRecordTime {}
impl RecordCommon for SysRecordTime {
    fn name(&self) -> String {
        String::from("Time")
    }
    fn id() -> i32 {
        *SYS_RECORD_ID.get_or_init(|| RecordId::of::<SysRecordTime>())
    }
}
impl Hash for SysRecordTime {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.target_frame_rate.hash(state);
        self.scaled_time.hash(state);
        self.unscaled_time.hash(state);
        self.frame_num.hash(state);
        self.unscaled_delta_time.hash(state);
        self.scaled_delta_time.hash(state);
        self.average_fps.hash(state);
    }
}
impl Eq for SysRecordTime {}

impl RecordOverride for SysRecordTime {
    fn apply(&mut self, field: &str, val: &str) {}
    fn get_state(&self) -> Vec<crate::FieldState> {
        vec![
            FieldState::new("scaled_time", self.scaled_time),
            FieldState::new("unscaled_time", self.unscaled_time),
            FieldState::new("scaled_delta_time", self.scaled_delta_time),
            FieldState::new("unscaled_delta_time", self.unscaled_delta_time),
            FieldState::new("frame_num", self.frame_num),
            FieldState::new("average_fps", self.average_fps),
        ]
    }
}
