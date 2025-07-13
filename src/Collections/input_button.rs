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

    pub fn update(&mut self, is_down: &bool) {
        self.went_down = *is_down && !self.is_down;
        self.went_up = !is_down && self.is_down;
        self.is_down = *is_down;
    }
}
