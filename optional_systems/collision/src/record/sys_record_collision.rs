use crate::facet::component_collider::CollisionSnapshot;
use curio_core::{FieldState, RecordCommon, RecordId, RecordOverride};
use std::sync::OnceLock;

static SYS_RECORD_ID: OnceLock<i32> = OnceLock::new();

#[derive(Default, Hash, Clone)]
pub struct SysRecordCollision {
    pub collisions: Vec<CollisionSnapshot>,
}
impl SysRecordCollision {}

impl RecordCommon for SysRecordCollision {
    fn name(&self) -> String {
        String::from("Collision")
    }
    fn id() -> i32 {
        *SYS_RECORD_ID.get_or_init(|| RecordId::of::<SysRecordCollision>())
    }
}
impl RecordOverride for SysRecordCollision {
    fn apply(&mut self, field: &str, val: &str) {}
    fn get_state(&self) -> Vec<FieldState> {
        vec![]
    }
}
