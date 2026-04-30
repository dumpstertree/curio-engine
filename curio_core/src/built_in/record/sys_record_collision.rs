use std::sync::OnceLock;

use crate::{
    built_in::facet::component_collider::CollisionSnapshot,
    system::{record_id::RecordId, system_game_state::RecordCommon},
};

static SYS_RECORD_ID: OnceLock<i32> = OnceLock::new();

#[derive(Default, Hash, Clone)]
pub struct SysRecordCollision {
    pub collisions: Vec<CollisionSnapshot>,
}
impl SysRecordCollision {}

impl RecordCommon for SysRecordCollision {
    fn id() -> i32 {
        *SYS_RECORD_ID.get_or_init(|| RecordId::of::<SysRecordCollision>())
    }
}
