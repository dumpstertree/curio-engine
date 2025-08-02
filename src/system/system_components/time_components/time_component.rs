use std::{alloc::System, sync::Arc, time::Instant};

use winit::keyboard::KeyCode;

use crate::system::system_game_states::state_gui::GuiWindow;
use crate::system::system_game_states::state_gui_debug::GUIState_Debug;
// use crate::system_adapters::adapter_system_gpu::CustomEvents;
use crate::Collections::game_state::GameState;
use crate::Collections::key_state::KeyState;
use crate::Collections::vector3::Vector3;
use crate::Collections::Color::Color;
use crate::{
    system::{
        system_component::ISystemComponent,
        system_components::{gameplay_components::gameplay_component_default::EngineCommands, time_component::time_component},
        system_game_states::{
            state_gui::{GUIState, GuiElement},
            state_time::TimeState,
        },
    },
    system_adapters::adapter_system_gpu::SystemGPU,
};

pub struct TimeComponent {
    instant: Instant,
    fps_average: Vec<f64>,
    timescale: f32,
    pause: bool,
}

impl TimeComponent {
    pub fn new() -> TimeComponent {
        TimeComponent {
            instant: Instant::now(),
            fps_average: Vec::new(),
            timescale: 1.0,
            pause: false,
        }
    }
}
impl time_component for TimeComponent {}
impl ISystemComponent for TimeComponent {
    fn order(&self) -> i32 {
        1000
    }
    fn init(&mut self, gs: &mut GameState) {}

    fn tick(&mut self, game_state: &mut GameState) -> &[EngineCommands] {
        println!("tick time staty");
        let cur_time = self.instant.elapsed().as_secs_f64();

        let fps = 1.0 / (cur_time - game_state.get_value2::<TimeState>().time);
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
        let pause_timescale = if self.pause { 0.0 } else { 1.0 };

        // edit the state
        game_state.edit::<TimeState>(|x| {
            x.average_fps = average_fps as i32;
            x.delta_time = (cur_time - x.time) as f32 * pause_timescale;
            x.time = cur_time;

            // update
            x.next_update = cur_time + (1.0 / x.target_frame_rate) as f64;
            x.frame_num = x.frame_num + 1;
        });

        &[]
    }
    fn debug(&mut self, gs: &mut GameState) {}

    fn render(&mut self, game_state: &mut GameState) -> &[EngineCommands] {
        // calculate if we tick this frame
        let cur_time = self.instant.elapsed().as_secs_f64();
        let nxt_time = game_state.get_value2::<TimeState>().next_update;

        // if do tick send event
        let do_tick = cur_time >= nxt_time;
        if do_tick {
            return &[EngineCommands::Redraw, EngineCommands::Tick];
        }
        // defualt
        return &[EngineCommands::Redraw];
    }
    fn input_keyboard(&mut self, gs: &mut GameState, key: winit::keyboard::KeyCode, key_state: crate::Collections::key_state::KeyState) {
        if key == KeyCode::KeyP && key_state == KeyState::Down {
            self.pause = !self.pause;
        }
    }
}
