use core::{
    Collections::{event_queue::EventQueue2, game_state::GameState},
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
    system::system_game_states::state_gui_debug::GUIStateDebug,
    system_adapters::adapter_system_gpu::SystemGPU,
};
use ecs_system::global_ecs_system;
use hecs::World;

#[global_ecs_system]
pub struct SystemDebugGuiScreen {}
impl SystemDebugGuiScreen {}
impl SystemDebugGuiScreen {
    pub fn new() -> Box<SystemDebugGuiScreen> {
        Box::new(SystemDebugGuiScreen {})
    }
}
impl ECSSystemEventless for SystemDebugGuiScreen {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn debug(&mut self, game_state: &mut GameState, _: &mut World, _: &mut EventQueue2) {
        // get gpu data
        let sys_config = SystemGPU::get_config();
        let sys_window = SystemGPU::get_window();

        // edit state
        game_state.edit::<GUIStateDebug>(|x| {
            x.append(format!("Resolution: ({}, {})", sys_config.width, sys_config.height));
            x.append(format!(
                "Screen Size: ({}, {})",
                sys_window.inner_size().width,
                sys_window.inner_size().height
            ));
        });
    }
}
