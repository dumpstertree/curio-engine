use winit::keyboard::KeyCode;

use crate::{system::system_component::ISystemComponent, Collections::key_state::KeyState};

pub struct InputComponentDefault {}

impl InputComponentDefault {
    pub fn new() -> InputComponentDefault {
        InputComponentDefault {}
    }
}
impl ISystemComponent for InputComponentDefault {
    fn order(&self) -> i32 {
        1000
    }
    fn init(&mut self, state: &mut crate::Window::state::State, gs: &mut crate::game_state::GameState) {
        println!("init input");
    }
    fn input_keyboard(&mut self, game_state: &mut crate::game_state::GameState, key: KeyCode, key_state: KeyState) {
        let mut input = game_state.get_input();
        match key {
            KeyCode::KeyW => {
                input.w.is_down = key_state == KeyState::Down;
            }
            KeyCode::KeyA => {
                input.a.is_down = key_state == KeyState::Down;
            }
            KeyCode::KeyS => {
                input.s.is_down = key_state == KeyState::Down;
            }
            KeyCode::KeyD => {
                input.d.is_down = key_state == KeyState::Down;
            }
            _ => {}
        }
        game_state.set_input(input);
    }
}
