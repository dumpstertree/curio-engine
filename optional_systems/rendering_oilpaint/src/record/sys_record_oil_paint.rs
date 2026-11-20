// The dab submission for one screen this frame. The paint habit refills it
// every tick from the world's PaintOil forms; the dab pass reads its own
// screen's instance. dabs are object-LOCAL — models is parallel bookkeeping
// of each dab's form matrix. The draw pass bakes them together until
// phase 4, where the matrix goes to the vertex shader as a second
// instance attribute and the CPU bake disappears.

use crate::Dab;
use curio_core::{FieldState, Matrix4x4, RecordOverride, RecordScope};
use record::record;

#[record(name = "OilPaint", ownership = RecordScope::Instance)]
pub struct SysRecordOilPaint {
    /// the dabs a painted form put down, in that form's object-local space
    pub dabs: Vec<Dab>,
    /// parallel to dabs — each dab's form matrix (one entry per dab so the
    /// GPU stream stays a straight 48 B + 64 B pair in phase 4)
    pub models: Vec<Matrix4x4>,
}

impl SysRecordOilPaint {
    pub fn default() -> SysRecordOilPaint {
        SysRecordOilPaint {
            dabs: Vec::new(),
            models: Vec::new(),
        }
    }
}

impl RecordOverride for SysRecordOilPaint {
    fn set_state(&mut self, _field: &str, _value: &str) {
        // nothing here is authored per composition — paint lives on the
        // forms (PaintOil), the record only carries what the habit made
    }
    fn get_state(&self) -> Vec<FieldState> {
        vec![
            FieldState::new("dabs", self.dabs.len()),
            FieldState::new("models", self.models.len()),
        ]
    }
}
