use crate::{
    system::system_game_states::state_gui_debug::GUIState_Debug, system_adapters::adapter_system_gpu::SystemGPU, Collections::game_state::GameState,
};
use ecs_system::ECSSystem;
use hecs::World;

use crate::{
    gameplay::ecs::component::{component_camera::Camera, component_transform::Transform},
    system::{system_components::gameplay_components::gameplay_component_default::ECSSystemEventless, system_game_states::state_camera::CameraState},
    system::{
        system_components::gameplay_components::gameplay_component_default::{EngineCommands, EventQueue},
        system_game_states::{state_colliders::StateCollider, state_collision::StateCollision, state_gizmos::GizmosState},
    },
};

#[derive(ECSSystem)]
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
    fn debug(&mut self, game_state: &mut GameState, world: &mut World, system_event_queue: &mut EventQueue<EngineCommands>) {
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
impl Default for SystemDebugGuiScreen {
    fn default() -> Self {
        Self {}
    }
}
