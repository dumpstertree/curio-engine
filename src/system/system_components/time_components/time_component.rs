use std::time::Instant;

use crate::{game_state::GameState, system::system_component::ISystemComponent, Window::state::State};

pub struct TimeComponent {
    instant: Instant,
}

impl TimeComponent {
    pub fn new() -> TimeComponent {
        TimeComponent { instant: Instant::now() }
    }
}
impl ISystemComponent for TimeComponent {
    fn order(&self) -> i32 {
        1000
    }
    fn init(&mut self, state: &mut crate::Window::state::State, gs: &mut crate::game_state::GameState) {}
    fn render(&mut self, state: &mut State, game_state: &mut GameState) {
        let mut t = game_state.get_time();
        // get cur time
        let cur_time = self.instant.elapsed().as_secs_f64();

        // is elapsed
        if cur_time >= t.next_update {
            t.delta_time = (cur_time - t.time) as f32;
            t.time = cur_time;

            t.should_update = true;
            // update
            t.next_update = t.next_update + (1.0 / t.target_frame_rate) as f64;
            t.frame_num = t.frame_num + 1;

            game_state.set_time(t);
        } else {
            t.should_update = false;

            game_state.set_time(t);
        }
    }
}
