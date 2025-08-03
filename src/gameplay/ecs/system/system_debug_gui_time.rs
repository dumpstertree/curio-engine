use crate::{
    system::system_components::gameplay_components::gameplay_component_default::{EngineCommands, EventQueue},
    system::system_game_states::{
        state_colliders::StateCollider, state_collision::StateCollision, state_gui_debug::GUIState_Debug, state_time::TimeState,
    },
    Collections::game_state::GameState,
};
use hecs::World;

use crate::system::system_components::gameplay_components::gameplay_component_default::ECSSystemEventless;

pub struct SystemDebugGuiTime {}
impl SystemDebugGuiTime {}
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

        // let fps = state_time.next_update - state_time.time

        game_state.edit::<GUIState_Debug>(|x| {
            x.append(format!("FPS: {} / Target FPS: {}", state_time.average_fps, state_time.target_frame_rate));
            x.append(format!("Scaled Time: {}", state_time.scaled_time));
            x.append(format!("Unscaled Time: {}", state_time.unscaled_time));
            x.append(format!("Frame Num: {}", state_time.frame_num));
        });
    }
}
