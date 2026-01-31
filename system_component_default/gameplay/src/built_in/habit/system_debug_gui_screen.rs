use curio_core::{
    built_in::record::{sys_record_debug::SysRecordDebug, sys_record_debug_gui::SysRecordDebugGui},
    collections::network_modes::NetworkModes,
    collections::{event_queue::EventQueue, game_state::GameState},
    system_adapters::adapter_system_gpu::SystemGPU,
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
    fn is_enabled(&mut self, game_state: &mut GameState) -> bool {
        game_state.get::<SysRecordDebug>().is_inspecting
    }
    fn run_on_instance(&mut self, _game_state: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all()
    }
}
impl Habit for Instance {
    fn tick(&mut self, game_state: &mut GameState, _: &mut Context3D, _: &mut EventQueue) {
        // get gpu data
        let sys_config = SystemGPU::get_config();
        let sys_window = SystemGPU::get_window();

        // edit state
        game_state.edit::<SysRecordDebugGui>(|x| {
            x.append(format!("Resolution: ({}, {})", sys_config.width, sys_config.height));
            x.append(format!("Screen Size: ({}, {})", sys_window.inner_size().width, sys_window.inner_size().height));
        });
    }
}
