use curio_core::{
    built_in::record::{state_debug::StateDebug, state_input::InputState},
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    input::key_code::ButtonCode,
};

use crate::{
    context_3d::Context3D,
    traits::{habit::Habit, scope::Scope},
};

#[derive(Default)]
pub struct Instance {}
impl Instance {
    pub fn new() -> Box<Instance> {
        Box::new(Instance {})
    }
}
impl Scope for Instance {
    fn is_enabled(&mut self, _game_state: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _game_state: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all_host()
    }
}
impl Habit for Instance {
    fn tick(&mut self, game_state: &mut GameState, _: &mut Context3D, _: &mut EventQueue) {
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
