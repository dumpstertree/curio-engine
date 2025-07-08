use crate::system::system_game_state::IState;

#[derive(Clone)]
pub struct TimeState {
    pub target_frame_rate: f32,
    pub next_update: f64,
    pub time: f64,
    pub frame_num: i64,
    pub delta_time: f32,
    pub should_update: bool,
}
impl TimeState {
    fn default() -> TimeState {
        TimeState {
            target_frame_rate: 120.0,
            next_update: 0.0,
            time: 0.0,
            frame_num: 0,
            delta_time: 0.0,
            should_update: false,
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
