use core::{
    collections::{event_queue::EventQueue2, game_state::GameState},
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
    system::system_game_states::{state_collision::StateCollision, state_gui_debug::GUIStateDebug},
};
use ecs_system::global_ecs_system;
use hecs::World;

#[global_ecs_system]
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
    fn debug(&mut self, game_state: &mut GameState, _: &mut World, _: &mut EventQueue2) {
        // get state
        let state_collision = game_state.get_value2::<StateCollision>();
        // edit state
        game_state.edit::<GUIStateDebug>(|x| {
            x.append(format!("Collision Count: {}", state_collision.collisions.len()));
        });
    }
}
