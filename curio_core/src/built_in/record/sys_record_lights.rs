use std::sync::OnceLock;

use crate::{
    graphics::light_uniform::DrawCallLight,
    system::{record_id::RecordId, system_game_state::RecordCommon},
};

static SYS_RECORD_ID: OnceLock<i32> = OnceLock::new();

#[derive(Default, Hash, Clone, PartialEq, Eq)]
pub struct SysRecordLights {
    pub all_lights: Vec<DrawCallLight>,
}

impl SysRecordLights {
    pub fn default() -> SysRecordLights {
        SysRecordLights { all_lights: Vec::new() }
    }
}
impl RecordCommon for SysRecordLights {
    fn id() -> i32 {
        *SYS_RECORD_ID.get_or_init(|| RecordId::of::<SysRecordLights>())
    }
}
