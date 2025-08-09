use crate::{gameplay::ecs::component::component_collider::CollisionSnapshot, system::system_game_state::IState};

#[derive(Clone)]
pub struct StateCollision {
    pub collisions: Vec<CollisionSnapshot>,
}
impl StateCollision {
    fn new() -> StateCollision {
        StateCollision { collisions: Vec::new() }
    }
}

impl IState<StateCollision> for StateCollision {
    fn id() -> i32 {
        85738
    }
    fn default() -> StateCollision {
        StateCollision::new()
    }
}
