use crate::game_state::GameState;
use crate::system::system_component::ISystemComponent;
use crate::Window::state::State;

pub trait input_component: ISystemComponent {}

const KEY: i32 = 900;
impl GameState {
    pub fn set_input(&mut self, state: InputState) {
        self.add(KEY, state);
    }
    pub fn get_input(&self) -> InputState {
        if !self.has_value(KEY) {
            return InputState::default();
        }
        let x = self.get_value::<InputState>(KEY);
        x.unwrap().clone()
    }
}

#[derive(Clone)]
pub struct InputState {
    pub w: InputButtonState,
    pub a: InputButtonState,
    pub s: InputButtonState,
    pub d: InputButtonState,
}

impl InputState {
    pub fn default() -> InputState {
        InputState {
            w: InputButtonState::default(),
            a: InputButtonState::default(),
            s: InputButtonState::default(),
            d: InputButtonState::default(),
        }
    }
}

#[derive(Clone)]
pub struct InputButtonState {
    pub went_down: bool,
    pub is_down: bool,
    pub went_up: bool,
}

impl InputButtonState {
    pub fn default() -> InputButtonState {
        InputButtonState {
            went_down: false,
            is_down: false,
            went_up: false,
        }
    }
}
