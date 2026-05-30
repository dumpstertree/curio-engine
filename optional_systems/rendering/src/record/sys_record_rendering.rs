use crate::DrawCall;
use curio_core::{FieldState, RecordOverride, StateOwnerships};
use record_serializable::record_serializable;

#[record_serializable(name = "Rendering", ownership = StateOwnerships::Instance)]
pub struct SysRecordRendering {
    pub draw_calls: Vec<DrawCall>,
}
impl RecordOverride for SysRecordRendering {
    fn apply(&mut self, _: &str, _: &str) {}
    fn get_state(&self) -> Vec<FieldState> {
        vec![FieldState::new("num_calls", self.draw_calls.len())]
    }
}
