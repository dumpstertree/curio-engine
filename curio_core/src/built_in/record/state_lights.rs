use crate::{collections::light_uniform::DrawCallLight, system::system_game_state::IState};

#[derive(Default, Hash, Clone, PartialEq, Eq)]
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
