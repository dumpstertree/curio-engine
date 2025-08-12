use crate::{
    input::{input_snapshot_mapped::PlayerInputSnapshot, input_snapshot_raw::RawInputSnapshot},
    system::system_game_state::IState,
};

#[derive(Clone)]
pub struct InputState {
    pub mapped: Vec<PlayerInputSnapshot>,
    pub raw: RawInputSnapshot,
}

impl InputState {
    pub fn default() -> InputState {
        InputState {
            mapped: Vec::new(),
            raw: RawInputSnapshot::new(),
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
