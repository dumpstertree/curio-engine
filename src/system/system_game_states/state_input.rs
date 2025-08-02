use crate::{
    system::system_game_state::IState,
    Collections::{
        input_button::InputButtonState,
        input_cursor::{self, InputAxisState},
    },
};

#[derive(Clone)]
pub struct InputState {
    pub cursor: InputAxisState,
    pub w: InputButtonState,
    pub a: InputButtonState,
    pub s: InputButtonState,
    pub d: InputButtonState,
    pub esc: InputButtonState,
    pub tab: InputButtonState,
    pub debug: InputButtonState,
}

impl InputState {
    pub fn default() -> InputState {
        InputState {
            cursor: InputAxisState::default(),
            w: InputButtonState::default(),
            a: InputButtonState::default(),
            s: InputButtonState::default(),
            d: InputButtonState::default(),
            esc: InputButtonState::default(),
            tab: InputButtonState::default(),
            debug: InputButtonState::default(),
        }
    }
}
impl IState<InputState> for InputState {
    fn default() -> InputState {
        InputState::default()
    }

    fn id() -> i32 {
        290873492
    }
}
