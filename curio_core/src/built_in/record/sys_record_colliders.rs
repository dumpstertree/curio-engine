use crate::{gameplay::ecs::component::component_collider::ColliderSnapshot, system::system_game_state::IState};

#[derive(Default, Hash, Clone)]
pub struct SysRecordCollider {
    pub colliders: Vec<ColliderSnapshot>,
}
impl SysRecordCollider {}

impl IState for SysRecordCollider {
    fn id() -> i32 {
        98341
    }
}
