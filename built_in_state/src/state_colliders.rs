use curio_core::{gameplay::ecs::component::component_collider::ColliderSnapshot, system::system_game_state::IState};

use macro_state::global_state;

#[derive(Hash)]
#[global_state]
pub struct StateCollider {
    pub colliders: Vec<ColliderSnapshot>,
}
impl StateCollider {
    fn new() -> StateCollider {
        StateCollider { colliders: Vec::new() }
    }
}

impl IState for StateCollider {
    fn id() -> i32 {
        98341
    }
}
