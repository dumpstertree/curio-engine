use core::{
    extensions::{extensions_f32::ExtensionsF32, extensions_f64::ExtensionsF64},
    system::system_game_state::IState,
};
use std::hash::Hash;

use macro_state::global_state;

#[global_state]
pub struct TimeState {
    pub target_frame_rate: f32,
    pub scaled_time: f64,
    pub unscaled_time: f64,
    pub frame_num: i64,
    pub unscaled_delta_time: f32,
    pub scaled_delta_time: f32,
    pub average_fps: i32,
}
impl TimeState {}
impl IState for TimeState {
    fn id() -> i32 {
        38345
    }
}
impl Hash for TimeState {
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
impl Eq for TimeState {}
