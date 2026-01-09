use built_in_state::{state_debug::StateDebug, state_gui_debug::GUIStateDebug};
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    system_adapters::adapter_system_gpu::SystemGPU,
};

use crate::{
    traits::{habit::Habit, scope::Scope},
    world_context_3d::WorldContext,
};

#[derive(Default)]
pub struct Instance {}
impl Instance {
    pub fn new() -> Box<Instance> {
        Box::new(Instance {})
    }
}
impl Scope for Instance {
    fn is_enabled(&mut self, game_state: &mut GameState) -> bool {
        game_state.get::<StateDebug>().is_inspecting
    }
    fn run_on_instance(&mut self, _game_state: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all()
    }
}
impl Habit for Instance {
    fn tick(&mut self, game_state: &mut GameState, _: &mut WorldContext, _: &mut EventQueue) {
        // get gpu data
        let sys_config = SystemGPU::get_config();
        let sys_window = SystemGPU::get_window();

        // edit state
        game_state.edit::<GUIStateDebug>(|x| {
            x.append(format!("Resolution: ({}, {})", sys_config.width, sys_config.height));
            x.append(format!("Screen Size: ({}, {})", sys_window.inner_size().width, sys_window.inner_size().height));
        });
    }
}
