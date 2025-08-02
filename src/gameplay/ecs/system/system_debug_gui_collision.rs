use crate::{
    system::system_components::gameplay_components::gameplay_component_default::{EngineCommands, EventQueue},
    system::system_game_states::{state_colliders::StateCollider, state_collision::StateCollision, state_gui_debug::GUIState_Debug},
    Collections::game_state::GameState,
};
use hecs::World;

use crate::system::system_components::gameplay_components::gameplay_component_default::ECSSystemEventless;

pub struct SystemDebugGuiCollisions {}
impl SystemDebugGuiCollisions {}
impl SystemDebugGuiCollisions {
    pub fn new() -> Box<SystemDebugGuiCollisions> {
        Box::new(SystemDebugGuiCollisions {})
    }
}
impl ECSSystemEventless for SystemDebugGuiCollisions {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn debug(&mut self, game_state: &mut GameState, world: &mut World, system_event_queue: &mut EventQueue<EngineCommands>) {
        // get state
        let state_collision = game_state.get_value2::<StateCollision>();
        // edit state
        game_state.edit::<GUIState_Debug>(|x| {
            x.append(format!("Collision Count: {}", state_collision.collisions.len()));
        });
    }
}
