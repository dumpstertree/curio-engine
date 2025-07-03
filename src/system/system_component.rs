use winit::keyboard::KeyCode;

use crate::game_state::GameState;
use crate::state::State;
use crate::Collections::key_state::KeyState;

pub trait ISystemComponent {
    fn order(&self) -> i32;
    fn init(&mut self, state: &mut State, gs: &mut GameState);
    fn quit(&mut self) {}
    fn resize(&mut self, width: f32, height: f32) {}
    fn render(&mut self, state: &mut State, gs: &mut GameState) {}
    fn input_mouse(&mut self) {}
    fn input_keyboard(&mut self, gs: &mut GameState, key: KeyCode, key_state: KeyState) {}
}
