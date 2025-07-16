use crate::{gameplay::ecs::component::component_collider::ColliderSnapshot, system::system_game_state::IState};

#[derive(Clone)]
pub struct StateCollider {
    pub colliders: Vec<ColliderSnapshot>,
}
impl StateCollider {
    fn new() -> StateCollider {
        StateCollider { colliders: Vec::new() }
    }
}

impl IState<StateCollider> for StateCollider {
    fn id() -> i32 {
        98341
    }
    fn default() -> StateCollider {
        StateCollider::new()
    }
}
