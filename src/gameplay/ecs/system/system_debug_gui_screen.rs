use crate::{
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
    system::system_game_states::state_gui_debug::GUIState_Debug,
    system_adapters::adapter_system_gpu::SystemGPU,
    Collections::{event_queue::EventQueue2, game_state::GameState},
};
use ecs_system::ECSSystem;
use hecs::World;


#[ECSSystem]
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
    fn debug(&mut self, game_state: &mut GameState, world: &mut World, system_event_queue: &mut EventQueue2) {
        // get gpu data
        let sys_config = SystemGPU::get_config();
        let sys_window = SystemGPU::get_window();

        // edit state
        game_state.edit::<GUIState_Debug>(|x| {
            x.append(format!("Resolution: ({}, {})", sys_config.width, sys_config.height));
            x.append(format!(
                "Screen Size: ({}, {})",
                sys_window.inner_size().width,
                sys_window.inner_size().height
            ));
        });
    }
}
