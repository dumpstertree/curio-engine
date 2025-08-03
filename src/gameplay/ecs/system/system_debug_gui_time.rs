use crate::{
    system::system_components::gameplay_components::gameplay_component_default::ECSSystemEventless,
    system::system_components::gameplay_components::gameplay_component_default::{EngineCommands, EventQueue},
    system::system_game_states::{state_gui_debug::GUIState_Debug, state_time::TimeState},
    Collections::game_state::GameState,
};
use ecs_system::ECSSystem;
use hecs::World;

#[derive(ECSSystem)]
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
    fn debug(&mut self, game_state: &mut GameState, world: &mut World, system_event_queue: &mut EventQueue<EngineCommands>) {
        // get state
        let state_time = game_state.get_value2::<TimeState>();

        game_state.edit::<GUIState_Debug>(|x| {
            x.append(format!("FPS: {} / Target FPS: {}", state_time.average_fps, state_time.target_frame_rate));
            x.append(format!("Scaled Time: {}", state_time.scaled_time));
            x.append(format!("Unscaled Time: {}", state_time.unscaled_time));
            x.append(format!("Frame Num: {}", state_time.frame_num));
        });
    }
}
impl Default for SystemDebugGuiTime {
    fn default() -> Self {
        Self {}
    }
}
