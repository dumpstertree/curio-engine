use built_in_state::{state_debug::StateDebug, state_gui_debug::GUIStateDebug};
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    // gameplay::{ecs::traits::ecs_system::ECSSystemEventless, world_context::WorldContext},
    system_adapters::adapter_system_gpu::SystemGPU,
};
// use ecs_system::global_ecs_system;

use crate::{traits::ecs_system::ECSSystemEventless, world_context::WorldContext};

// #[global_ecs_system]
#[derive(Default)]

pub struct SystemDebugGuiScreen {}
impl SystemDebugGuiScreen {}
impl SystemDebugGuiScreen {
    pub fn new() -> Box<SystemDebugGuiScreen> {
        Box::new(SystemDebugGuiScreen {})
    }
}
impl ECSSystemEventless for SystemDebugGuiScreen {
    fn is_enabled(&mut self, game_state: &mut GameState, _: &mut WorldContext) -> bool {
        game_state.get::<StateDebug>().is_inspecting
    }
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
