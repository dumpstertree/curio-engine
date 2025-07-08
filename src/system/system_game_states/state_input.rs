use crate::{system::system_game_state::IState, Collections::input_button::InputButtonState};

#[derive(Clone)]
pub struct InputState {
    pub w: InputButtonState,
    pub a: InputButtonState,
    pub s: InputButtonState,
    pub d: InputButtonState,
    pub esc: InputButtonState,
    pub tab: InputButtonState,
}

impl InputState {
    pub fn default() -> InputState {
        InputState {
            w: InputButtonState::default(),
            a: InputButtonState::default(),
            s: InputButtonState::default(),
            d: InputButtonState::default(),
            esc: InputButtonState::default(),
            tab: InputButtonState::default(),
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
