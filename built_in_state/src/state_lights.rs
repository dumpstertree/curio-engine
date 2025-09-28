use core::{
    collections::{light_uniform::DrawCallLight, matrix4x4::Matrix4x4},
    input::{input_snapshot_mapped::PlayerInputSnapshot, input_snapshot_raw::RawInputSnapshot},
    system::system_game_state::IState,
};

use macro_state::global_state;

#[global_state]
pub struct StateLights {
    pub all_lights: Vec<DrawCallLight>,
}

impl StateLights {
    pub fn default() -> StateLights {
        StateLights { all_lights: Vec::new() }
    }
}
impl IState for StateLights {
    fn id() -> i32 {
        0983543847
    }
}
