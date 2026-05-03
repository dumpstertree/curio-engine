use crate::{built_in::facet::component_collider::ColliderSnapshot, system::record_id::RecordId, RecordCommon};
use std::sync::OnceLock;

static SYS_RECORD_ID: OnceLock<i32> = OnceLock::new();

#[derive(Default, Hash, Clone)]
pub struct SysRecordCollider {
    pub colliders: Vec<ColliderSnapshot>,
}
impl SysRecordCollider {}

impl RecordCommon for SysRecordCollider {
    fn id() -> i32 {
        *SYS_RECORD_ID.get_or_init(|| RecordId::of::<SysRecordCollider>())
    }
}
