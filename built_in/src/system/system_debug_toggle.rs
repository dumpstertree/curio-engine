use built_in_state::{state_debug::StateDebug, state_input::InputState};
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
    input::key_code::ButtonCode,
};
use ecs_system::global_ecs_system;
use hecs::World;

#[global_ecs_system]
pub struct SystemDebugToggle {}
impl SystemDebugToggle {
    pub fn new() -> Box<SystemDebugToggle> {
        Box::new(SystemDebugToggle {})
    }
}

impl ECSSystemEventless for SystemDebugToggle {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
    fn tick(&mut self, game_state: &mut GameState, _: &mut World, _: &mut EventQueue) {
        // get state
        let state_input = game_state.get::<InputState>();

        // get input button
        let debug_button = state_input.raw.get_button(&ButtonCode::Backquote);
        if debug_button.went_up {
            // flip the toggle
            game_state.edit::<StateDebug>(|x| {
                x.is_inspecting = !x.is_inspecting;
            });
        }
    }
}
