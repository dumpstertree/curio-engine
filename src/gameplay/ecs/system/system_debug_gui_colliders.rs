use crate::{
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
    system::system_game_states::{state_colliders::StateCollider, state_collision::StateCollision, state_gui_debug::GUIState_Debug},
    Collections::{event_queue::EventQueue2, game_state::GameState},
};
use ecs_system::ECSSystem;
use hecs::World;

#[ECSSystem]
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
    fn debug(&mut self, game_state: &mut GameState, world: &mut World, system_event_queue: &mut EventQueue2) {
        // get state
        let state_collider = game_state.get_value2::<StateCollider>();
        // edit state
        game_state.edit::<GUIState_Debug>(|x| {
            x.append(format!("Collider Count: {}", state_collider.colliders.len()));
        });
    }
}
