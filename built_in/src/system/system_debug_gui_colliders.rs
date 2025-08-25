use built_in_state::{state_colliders::StateCollider, state_gui_debug::GUIStateDebug};
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};
use ecs_system::global_ecs_system;
use hecs::World;

#[global_ecs_system]
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
    fn debug(&mut self, game_state: &mut GameState, _: &mut World, _: &mut EventQueue) {
        // get state
        let state_collider = game_state.get_value2::<StateCollider>();
        // edit state
        game_state.edit::<GUIStateDebug>(|x| {
            x.append(format!("Collider Count: {}", state_collider.colliders.len()));
        });
    }
}
