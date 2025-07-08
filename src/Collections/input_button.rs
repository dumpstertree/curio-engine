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
