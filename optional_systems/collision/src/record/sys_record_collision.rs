use crate::facet::component_collider::CollisionSnapshot;
use curio_core::{FieldState, RecordCommon, SequentialRecordUIDs, RecordOverride};
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
        *SYS_RECORD_ID.get_or_init(|| SequentialRecordUIDs::of::<SysRecordCollision>())
    }
}
impl RecordOverride for SysRecordCollision {
    fn set_state(&mut self, _field: &str, _val: &str) {}
    fn get_state(&self) -> Vec<FieldState> {
        vec![]
    }
}
