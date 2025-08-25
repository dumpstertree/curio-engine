use core::system::system_game_state::IState;

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
