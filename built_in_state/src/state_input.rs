use curio_core::{
    input::{input_snapshot_mapped::PlayerInputSnapshot, input_snapshot_raw::RawInputSnapshot},
    system::system_game_state::IState,
};

use macro_state::global_state;

#[derive(Hash, Eq)]
#[global_state]
pub struct InputState {
    pub mapped: Vec<PlayerInputSnapshot>,
    pub raw: RawInputSnapshot,
}

impl InputState {
    pub fn default() -> InputState {
        InputState { mapped: Vec::new(), raw: RawInputSnapshot::new() }
    }
}
impl IState for InputState {
    fn id() -> i32 {
        290873492
    }
}
