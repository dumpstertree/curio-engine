use crate::{
    system::system_components::gameplay_components::gameplay_component_default::{EngineCommands, EventQueue},
    system::system_game_states::{state_colliders::StateCollider, state_collision::StateCollision, state_gui_debug::GUIState_Debug},
    Collections::game_state::GameState,
};
use ecs_system::ECSSystem;
use hecs::World;

use crate::system::system_components::gameplay_components::gameplay_component_default::ECSSystemEventless;

#[derive(ECSSystem)]
pub struct SystemDebugGuiColliders {}
impl SystemDebugGuiColliders {}
impl SystemDebugGuiColliders {
    pub fn new() -> Box<SystemDebugGuiColliders> {
        Box::new(SystemDebugGuiColliders {})
    }
}
impl ECSSystemEventless for SystemDebugGuiColliders {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn debug(&mut self, game_state: &mut GameState, world: &mut World, system_event_queue: &mut EventQueue<EngineCommands>) {
        // get state
        let state_collider = game_state.get_value2::<StateCollider>();
        // edit state
        game_state.edit::<GUIState_Debug>(|x| {
            x.append(format!("Collider Count: {}", state_collider.colliders.len()));
        });
    }
}
impl Default for SystemDebugGuiColliders {
    fn default() -> Self {
        Self {}
    }
}
