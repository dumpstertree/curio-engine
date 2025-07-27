use std::{alloc::System, sync::Arc, time::Instant};

use crate::system::system_game_states::state_gui::GuiWindow;
// use crate::system_adapters::adapter_system_gpu::CustomEvents;
use crate::Collections::game_state::GameState;
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
}

impl TimeComponent {
    pub fn new() -> TimeComponent {
        TimeComponent {
            instant: Instant::now(),
            fps_average: Vec::new(),
        }
    }
}
impl time_component for TimeComponent {}
impl ISystemComponent for TimeComponent {
    fn order(&self) -> i32 {
        1000
    }
    fn init(&mut self, gs: &mut GameState) {}
    fn debug(&mut self, game_state: &mut GameState) {
        let state_time = game_state.get_value2::<TimeState>();

        game_state.edit::<GUIState>(|x| {
            let mut total = 0.0;
            for fps in &self.fps_average {
                total = total + fps;
            }
            let average_fps = (total / (self.fps_average.len() as f64))
                .round()
                .to_string();

            x.guis.push(
                GuiWindow::new(Vector3::zero(), Vector3::zero())
                    .add(GuiElement::new_label(
                        format!("FPS: {} / Target FPS: {}", average_fps, state_time.target_frame_rate),
                        18.0,
                        Color::get_black(),
                    ))
                    .add(GuiElement::new_label(format!("Time: {}", state_time.time), 18.0, Color::get_black()))
                    .to_owned(),
            );
        });
    }
    fn render(&mut self, game_state: &mut GameState) -> &[EngineCommands] {
        let window = SystemGPU::get_window();
        window.request_redraw();

        let cur_time = self.instant.elapsed().as_secs_f64();
        let nxt_time = game_state.get_value2::<TimeState>().next_update;

        let do_tick = cur_time >= nxt_time;
        let fps = 1.0 / (cur_time - game_state.get_value2::<TimeState>().time);
        self.fps_average.push(fps);

        while self.fps_average.len() > 5 {
            self.fps_average.remove(0);
        }

        if do_tick {
            // edit the state
            game_state.edit::<TimeState>(|x| {
                x.delta_time = (cur_time - x.time) as f32;
                x.time = cur_time;

                // update
                x.next_update = cur_time + (1.0 / x.target_frame_rate) as f64;
                x.frame_num = x.frame_num + 1;

                x.should_update = do_tick;
            });
        }
        if do_tick {
            return &[EngineCommands::Tick];
        }
        return &[];
    }
}
