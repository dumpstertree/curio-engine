use crate::{
    gameplay::ecs::component::{
        component_collider::{ColliderSnapshot, CollisionSnapshot},
        component_colliders::component_collider_sphere::ComponentColliderSphere,
        component_transform::Transform,
    },
    system::system_game_states::{state_colliders::StateCollider, state_collision::StateCollision},
    Collections::game_state::GameState,
};
use hecs::World;

use crate::system::system_components::gameplay_components::gameplay_component_default::ECSSystemEventless;

pub struct SystemColliderBoxUpdateState {}
impl SystemColliderBoxUpdateState {
    pub fn new() -> Box<SystemColliderBoxUpdateState> {
        Box::new(SystemColliderBoxUpdateState {})
    }
}
impl ECSSystemEventless for SystemColliderBoxUpdateState {
    fn is_enabled(&mut self, game_state: &mut GameState, world: &mut World) -> bool {
        true
    }

    fn will_tick(&mut self, game_state: &mut GameState, world: &mut World) {
        let state = game_state.get_value2::<StateCollision>();
        for (_, collider) in world.query::<&mut ComponentColliderSphere>().iter() {
            let mut collision = Vec::<CollisionSnapshot>::new();

            for c in state.collisions.iter() {
                let is_same = c.collider_a.guid == collider.guid;
                if is_same {
                    collision.push(c.clone());
                }
            }
            collider.collisions = collision;
        }
    }
    fn did_tick(&mut self, game_state: &mut GameState, world: &mut World) {
        let mut state = game_state.get_value2::<StateCollider>();
        for (_, (collider, transform)) in world
            .query::<(&ComponentColliderSphere, &Transform)>()
            .iter()
        {
            state
                .colliders
                .push(ColliderSnapshot::new(0, transform.get_matrix(), collider.get_shape()));
        }
        game_state.set_value2::<StateCollider>(state);
    }
}
