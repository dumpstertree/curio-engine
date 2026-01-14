use curio_core::{collections::light_uniform::DrawCallLight, system::system_game_state::IState};

use macro_state::global_state;

#[derive(Hash, Eq)]
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
