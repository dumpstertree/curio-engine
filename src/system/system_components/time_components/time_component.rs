use std::time::Instant;

use crate::system::{
    system_component::ISystemComponent,
    system_components::{gameplay_components::gameplay_component_default::EngineCommands, time_component::time_component},
    system_game_states::state_time::TimeState,
};
use crate::Collections::game_state::GameState;

pub struct TimeComponent {
    instant: Instant,
}

impl TimeComponent {
    pub fn new() -> TimeComponent {
        TimeComponent { instant: Instant::now() }
    }
}
impl time_component for TimeComponent {}
impl ISystemComponent for TimeComponent {
    fn order(&self) -> i32 {
        1000
    }
    fn init(&mut self, gs: &mut GameState) {}
    fn render(&mut self, game_state: &mut GameState) -> &[EngineCommands] {
        let mut t = game_state.get_value2::<TimeState>();
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
        } else {
            t.should_update = false;
        }
        game_state.set_value2::<TimeState>(t);

        return &[];
    }
}
