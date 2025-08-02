use winit::event::WindowEvent;
use winit::keyboard::KeyCode;

use crate::system::system_components::gameplay_components::gameplay_component_default::EngineCommands;
// use crate::system_adapters::adapter_system_gpu::CustomEvents;
use crate::Collections::game_state::GameState;
use crate::Collections::key_state::KeyState;
use crate::Collections::vector3::Vector3;

pub trait ISystemComponent {
    fn render(&mut self, gs: &mut GameState) -> &[EngineCommands] {
        &[]
    }
    fn order(&self) -> i32;
    fn init(&mut self, gs: &mut GameState);
    fn quit(&mut self) {}
    fn resize(&mut self, width: f32, height: f32) {}
    fn tick(&mut self, gs: &mut GameState) -> &[EngineCommands] {
        return &[];
    }
    fn debug(&mut self, gs: &mut GameState) {}
    fn input_mouse(&mut self) {}
    fn input_mouse_position(&mut self, gs: &mut GameState, position: Vector3) {}
    fn input_keyboard(&mut self, gs: &mut GameState, key: KeyCode, key_state: KeyState) {}
    fn raw_event(&mut self, event: WindowEvent) {}
}
