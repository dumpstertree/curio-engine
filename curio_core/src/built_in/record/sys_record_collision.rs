use crate::{built_in::facet::component_collider::CollisionSnapshot, system::system_game_state::RecordCommon};

#[derive(Default, Hash, Clone)]
pub struct SysRecordCollision {
    pub collisions: Vec<CollisionSnapshot>,
}
impl SysRecordCollision {}

impl RecordCommon for SysRecordCollision {
    fn id() -> i32 {
        102
    }
}
