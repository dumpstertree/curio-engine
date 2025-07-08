use crate::Collections::game_state::GameState;
use winit::keyboard::KeyCode;

use crate::{
    system::{
        system_component::ISystemComponent,
        system_components::input_component::input_component,
        system_game_states::{state_input::InputState, state_time::TimeState},
    },
    Collections::key_state::KeyState,
};

pub struct InputComponentDefault {}

impl InputComponentDefault {
    pub fn new() -> InputComponentDefault {
        InputComponentDefault {}
    }
}
impl input_component for InputComponentDefault {}
impl ISystemComponent for InputComponentDefault {
    fn order(&self) -> i32 {
        1000
    }
    fn init(&mut self, gs: &mut GameState) {
        println!("init input");
    }
    fn input_keyboard(&mut self, game_state: &mut GameState, key: KeyCode, key_state: KeyState) {
        let time = game_state.get_value2::<TimeState>();
        if !time.should_update {
            return;
        }
        let mut input = game_state.get_value2::<InputState>();
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
            KeyCode::Escape => {
                input.esc.is_down = key_state == KeyState::Down;
            }
            KeyCode::Tab => {
                input.tab.is_down = key_state == KeyState::Down;
            }
            _ => {}
        }
        game_state.set_value2::<InputState>(input);
    }
}
