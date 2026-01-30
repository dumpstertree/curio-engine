use std::hash::Hash;

use crate::{
    extensions::{extensions_f32::ExtensionsF32, extensions_f64::ExtensionsF64},
    system::system_game_state::IState,
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
impl IState for SysRecordTime {
    fn id() -> i32 {
        38345
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
