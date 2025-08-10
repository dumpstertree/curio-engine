use crate::events::engine_commands::EngineCommands;
use crate::Collections::event_queue::EventQueue2;
use crate::Collections::game_state::GameState;
use crate::Collections::key_state::KeyState;
use crate::Collections::vector3::Vector3;
use winit::event::{MouseButton, WindowEvent};
use winit::keyboard::KeyCode;

pub trait SystemComponent {
    fn render(&mut self, _: &mut GameState) -> &[EngineCommands] {
        &[]
    }
    fn order(&self) -> i32;
    fn init(&mut self, gs: &mut GameState);
    fn quit(&mut self) {}
    fn resize(&mut self, _: f32, _: f32) {}
    fn tick(&mut self, _: &mut GameState, _: &mut EventQueue2) {}
    fn debug(&mut self, _: &mut GameState, _: &mut EventQueue2) {}
    fn input_mouse(&mut self, _: MouseButton, _: KeyState) {}
    fn input_mouse_position(&mut self, _: &mut GameState, _: Vector3) {}
    fn input_keyboard(&mut self, _: &mut GameState, _: KeyCode, _: KeyState) {}
    fn raw_event(&mut self, _: WindowEvent) {}
}
