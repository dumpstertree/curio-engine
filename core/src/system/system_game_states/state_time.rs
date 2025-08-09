use crate::system::system_game_state::IState;

#[derive(Clone)]
pub struct TimeState {
    pub target_frame_rate: f32,
    pub scaled_time: f64,
    pub unscaled_time: f64,
    pub frame_num: i64,
    pub unscaled_delta_time: f32,
    pub scaled_delta_time: f32,
    pub average_fps: i32,
}
impl TimeState {
    fn default() -> TimeState {
        TimeState {
            target_frame_rate: 60.0,
            scaled_time: 0.0,
            unscaled_time: 0.0,
            frame_num: 0,
            scaled_delta_time: 0.0,
            unscaled_delta_time: 0.0,
            average_fps: 0,
        }
    }
}
impl IState<TimeState> for TimeState {
    fn default() -> TimeState {
        TimeState::default()
    }

    fn id() -> i32 {
        38345
    }
}
