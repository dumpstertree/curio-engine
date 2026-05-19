use serde::{Deserialize, Serialize};

/// Stored data data representing the current and change in the state of an Button since last poll
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ButtonState {
    pub went_down: bool,
    pub is_down: bool,
    pub went_up: bool,
}

impl ButtonState {
    pub fn update(&mut self, is_down: &bool) {
        self.went_down = *is_down && !self.is_down;
        self.went_up = !is_down && self.is_down;
        self.is_down = *is_down;
    }
}
