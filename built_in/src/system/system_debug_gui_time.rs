use core::{
    Collections::{event_queue::EventQueue2, game_state::GameState},
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
    system::system_game_states::{state_gui_debug::GUIStateDebug, state_time::TimeState},
};
use ecs_system::global_ecs_system;
use hecs::World;

#[global_ecs_system]
pub struct SystemDebugGuiTime {}
impl SystemDebugGuiTime {
    pub fn new() -> Box<SystemDebugGuiTime> {
        Box::new(SystemDebugGuiTime {})
    }
}

impl ECSSystemEventless for SystemDebugGuiTime {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn debug(&mut self, game_state: &mut GameState, _: &mut World, _: &mut EventQueue2) {
        // get state
        let state_time = game_state.get_value2::<TimeState>();

        game_state.edit::<GUIStateDebug>(|x| {
            x.append(format!("FPS: {} / Target FPS: {}", state_time.average_fps, state_time.target_frame_rate));
            x.append(format!("Scaled Time: {}", state_time.scaled_time));
            x.append(format!("Unscaled Time: {}", state_time.unscaled_time));
            x.append(format!("Frame Num: {}", state_time.frame_num));
        });
    }
}
