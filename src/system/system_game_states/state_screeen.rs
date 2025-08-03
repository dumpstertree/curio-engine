use crate::{system::system_game_state::IState, system_adapters::adapter_system_gpu::SystemGPU};

#[derive(Clone)]
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
impl IState<StateScreen> for StateScreen {
    fn id() -> i32 {
        464
    }
    fn default() -> StateScreen {
        let window = SystemGPU::get_window();
        StateScreen::new(window.inner_size().width as i32, window.inner_size().height as i32)
    }
}
