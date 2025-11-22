use built_in_state::{state_collision::StateCollision, state_debug::StateDebug, state_gui_debug::GUIStateDebug};
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
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
    fn is_enabled(&mut self, game_state: &mut GameState, _: &mut World) -> bool {
        game_state.get::<StateDebug>().is_inspecting
    }
    fn tick(&mut self, game_state: &mut GameState, _: &mut World, _: &mut EventQueue) {
        // get state
        let state_collision = game_state.get::<StateCollision>();
        // edit state
        game_state.edit::<GUIStateDebug>(|x| {
            x.append(format!("Collision Count: {}", state_collision.collisions.len()));
        });
    }
}
