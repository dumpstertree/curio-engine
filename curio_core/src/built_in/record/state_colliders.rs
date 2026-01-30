use crate::{gameplay::ecs::component::component_collider::ColliderSnapshot, system::system_game_state::IState};

#[derive(Default, Hash, Clone)]
pub struct StateCollider {
    pub colliders: Vec<ColliderSnapshot>,
}
impl StateCollider {}

impl IState for StateCollider {
    fn id() -> i32 {
        98341
    }
}
