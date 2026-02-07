use crate::{graphics::light_uniform::DrawCallLight, system::system_game_state::IState};

#[derive(Default, Hash, Clone, PartialEq, Eq)]
pub struct SysRecordLights {
    pub all_lights: Vec<DrawCallLight>,
}

impl SysRecordLights {
    pub fn default() -> SysRecordLights {
        SysRecordLights { all_lights: Vec::new() }
    }
}
impl IState for SysRecordLights {
    fn id() -> i32 {
        0983543847
    }
}
