use crate::{gameplay::ecs::component::component_collider::ColliderState, system::system_game_state::IState};

#[derive(Clone)]
pub struct StateCollider {
    pub colliders: Vec<ColliderState>,
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
