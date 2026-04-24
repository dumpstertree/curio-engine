use crate::{built_in::facet::component_collider::ColliderSnapshot, system::system_game_state::RecordCommon};

#[derive(Default, Hash, Clone)]
pub struct SysRecordCollider {
    pub colliders: Vec<ColliderSnapshot>,
}
impl SysRecordCollider {}

impl RecordCommon for SysRecordCollider {
    fn id() -> i32 {
        98341
    }
}
