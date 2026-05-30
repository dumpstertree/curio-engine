use crate::DrawCallLight;
use curio_core::{FieldState, RecordOverride, StateOwnerships};
use record_serializable::record_serializable;

#[record_serializable(name = "Lights", ownership = StateOwnerships::Instance)]
pub struct SysRecordLights {
    pub all_lights: Vec<DrawCallLight>,
}

impl SysRecordLights {
    pub fn default() -> SysRecordLights {
        SysRecordLights { all_lights: Vec::new() }
    }
}
impl RecordOverride for SysRecordLights {
    fn apply(&mut self, _: &str, _: &str) {}
    fn get_state(&self) -> Vec<FieldState> {
        vec![
            FieldState::new("lights", &self.all_lights), //
        ]
    }
}
