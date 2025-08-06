use std::time::Instant;


use crate::events::engine_commands::EngineCommands;
use crate::system::system_game_states::state_debug::StateDebug;
use crate::Collections::event_queue::EventQueue2;
// use crate::system_adapters::adapter_system_gpu::CustomEvents;
use crate::Collections::game_state::GameState;
use crate::system::{
        system_component::ISystemComponent,
        system_components::time_component::time_component,
        system_game_states::state_time::TimeState,
    };

pub struct TimeComponent {
    instant: Instant,
    fps_average: Vec<f64>,
    timescale: f32,
    next_update: f64,
}

impl TimeComponent {
    pub fn new() -> TimeComponent {
        TimeComponent {
            instant: Instant::now(),
            fps_average: Vec::new(),
            timescale: 1.0,
            next_update: 0.0,
        }
    }
}
impl time_component for TimeComponent {}
impl ISystemComponent for TimeComponent {
    fn order(&self) -> i32 {
        1000
    }
    fn init(&mut self, gs: &mut GameState) {}

    fn tick(&mut self, game_state: &mut GameState, system_event_queue: &mut EventQueue2) {
        let state_debug = game_state.get_value2::<StateDebug>();

        let cur_time = self.instant.elapsed().as_secs_f64();

        let fps = 1.0 / (cur_time - game_state.get_value2::<TimeState>().unscaled_time);
        self.fps_average.push(fps);

        while self.fps_average.len() > 5 {
            self.fps_average.remove(0);
        }

        let mut total = 0.0;
        for fps in &self.fps_average {
            total = total + fps;
        }
        let average_fps = (total / (self.fps_average.len() as f64)).round();

        // is paused
        let pause_timescale = if state_debug.is_paused { 0.0 } else { 1.0 };

        // edit the state
        game_state.edit::<TimeState>(|x| {
            x.frame_num = x.frame_num + 1;
            x.average_fps = average_fps as i32;
            // delta time
            x.unscaled_delta_time = (cur_time - x.unscaled_time) as f32;
            x.scaled_delta_time = x.unscaled_delta_time * pause_timescale;

            // time
            x.scaled_time = x.scaled_time + x.scaled_delta_time as f64;
            x.unscaled_time = cur_time;
        });
    }

    fn render(&mut self, game_state: &mut GameState) -> &[EngineCommands] {
        // get state
        let state_time = game_state.get_value2::<TimeState>();

        // calculate if we tick this frame
        let cur_time = self.instant.elapsed().as_secs_f64();
        let nxt_time = self.next_update;

        // if do tick send event
        let do_tick = cur_time >= nxt_time;
        if do_tick {
            // set next update
            self.next_update = (1.0 / state_time.target_frame_rate) as f64;

            // do tick
            return &[EngineCommands::Redraw, EngineCommands::Tick];
        }
        // defualt
        return &[EngineCommands::Redraw];
    }
}
