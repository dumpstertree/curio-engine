use core::system::system_game_state::IState;

use macro_state::global_state;

#[global_state]
pub struct StateScreen {
    width: i32,
    height: i32,
}
impl StateScreen {
    pub fn width(&self) -> i32 {
        self.width
    }
    pub fn height(&self) -> i32 {
        self.height
    }
    pub fn new<'a>(width: i32, height: i32) -> StateScreen {
        StateScreen { width, height }
    }
}
impl IState for StateScreen {
    fn id() -> i32 {
        464
    }
}
