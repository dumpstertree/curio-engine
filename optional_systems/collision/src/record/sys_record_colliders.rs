use curio_core::{FieldState, RecordCommon, RecordId, RecordOverride};
use std::sync::OnceLock;

use crate::ColliderSnapshot;

static SYS_RECORD_ID: OnceLock<i32> = OnceLock::new();

#[derive(Default, Hash, Clone)]
pub struct SysRecordCollider {
    pub colliders: Vec<ColliderSnapshot>,
}
impl SysRecordCollider {}

impl RecordCommon for SysRecordCollider {
    fn name(&self) -> String {
        String::from("Collider")
    }
    fn id() -> i32 {
        *SYS_RECORD_ID.get_or_init(|| RecordId::of::<SysRecordCollider>())
    }
}
impl RecordOverride for SysRecordCollider {
    fn set_state(&mut self, _field: &str, _val: &str) {}
    fn get_state(&self) -> Vec<FieldState> {
        vec![]
    }
}
