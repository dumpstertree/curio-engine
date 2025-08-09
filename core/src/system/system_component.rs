use crate::events::engine_commands::EngineCommands;
use crate::Collections::event_queue::EventQueue2;
use crate::Collections::game_state::GameState;
use crate::Collections::key_state::KeyState;
use crate::Collections::vector3::Vector3;
use winit::event::{MouseButton, WindowEvent};
use winit::keyboard::KeyCode;

pub trait SystemComponent {
    fn render(&mut self, gs: &mut GameState) -> &[EngineCommands] {
        &[]
    }
    fn order(&self) -> i32;
    fn init(&mut self, gs: &mut GameState);
    fn quit(&mut self) {}
    fn resize(&mut self, width: f32, height: f32) {}
    fn tick(&mut self, gs: &mut GameState, system_event_queue: &mut EventQueue2) {}
    fn debug(&mut self, gs: &mut GameState, system_event_queue: &mut EventQueue2) {}
    fn input_mouse(&mut self, key: MouseButton, key_state: KeyState) {}
    fn input_mouse_position(&mut self, gs: &mut GameState, position: Vector3) {}
    fn input_keyboard(&mut self, gs: &mut GameState, key: KeyCode, key_state: KeyState) {}
    fn raw_event(&mut self, event: WindowEvent) {}
}
