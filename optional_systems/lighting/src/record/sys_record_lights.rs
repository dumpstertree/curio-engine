use crate::DrawCallLight;
use curio_core::{FieldState, RecordOverride, RecordScope};
use record::record;

#[record(name = "Lights", ownership = RecordScope::Instance)]
pub struct SysRecordLights {
    pub all_lights: Vec<DrawCallLight>,
}

impl SysRecordLights {
    pub fn default() -> SysRecordLights {
        SysRecordLights { all_lights: Vec::new() }
    }
}
impl RecordOverride for SysRecordLights {
    fn set_state(&mut self, _: &str, _: &str) {}
    fn get_state(&self) -> Vec<FieldState> {
        vec![
            FieldState::new("lights", &self.all_lights), //
        ]
    }
}
