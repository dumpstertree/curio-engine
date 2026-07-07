use std::rc::Rc;

use ctor;
use curio_core::{CurioNetwork, FieldState, Ledger, RecordOverride, RecordScope};
use record_serializable::record_serializable;

#[record_serializable(name = "Time", ownership = RecordScope::Instance)]
pub struct SysRecordTime {
    pub target_frame_rate: f32,
    pub scaled_time: f64,
    pub unscaled_time: f64,
    pub frame_num: i64,
    pub unscaled_delta_time: f32,
    pub scaled_delta_time: f32,
    pub average_fps: i32,
}
impl RecordOverride for SysRecordTime {
    fn set_state(&mut self, _field: &str, _val: &str) {}
    fn get_state(&self) -> Vec<FieldState> {
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

pub trait ExtensionsLedger {
    fn time(&self) -> Rc<SysRecordTime>;
}

impl ExtensionsLedger for Ledger {
    fn time(&self) -> Rc<SysRecordTime> {
        self.read::<SysRecordTime>()
    }
}
