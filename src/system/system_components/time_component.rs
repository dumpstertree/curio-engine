use crate::game_state::GameState;
use crate::system::system_component::ISystemComponent;
use crate::Window::state::State;

pub trait time_component: ISystemComponent {}

const KEY: i32 = 987;
impl GameState {
    pub fn set_time(&mut self, state: TimeState) {
        self.add(KEY, state);
    }
    pub fn get_time(&self) -> TimeState {
        if !self.has_value(KEY) {
            return TimeState::default();
        }
        let x = self.get_value::<TimeState>(KEY);
        x.unwrap().clone()
    }
}

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
            target_frame_rate: 30.0,
            next_update: 0.0,
            time: 0.0,
            frame_num: 0,
            delta_time: 0.0,
            should_update: false,
        }
    }
}
