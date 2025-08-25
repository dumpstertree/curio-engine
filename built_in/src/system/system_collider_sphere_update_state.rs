use built_in_state::{state_colliders::StateCollider, state_collision::StateCollision, state_gizmos::GizmosState};
use core::{
    collections::{color::Color, event_queue::EventQueue, game_state::GameState, gizmo::Gizmo},
    gameplay::ecs::{
        component::component_collider::{ColliderSnapshot, CollisionSnapshot},
        traits::ecs_system::ECSSystemEventless,
    },
};
use ecs_system::global_ecs_system;
use hecs::World;

use crate::component::{component_colliders::component_collider_sphere::ComponentColliderSphere, component_transform::Transform};

#[global_ecs_system]
pub struct SystemColliderBoxUpdateState {}
impl SystemColliderBoxUpdateState {
    pub fn new() -> Box<SystemColliderBoxUpdateState> {
        Box::new(SystemColliderBoxUpdateState {})
    }
}
impl ECSSystemEventless for SystemColliderBoxUpdateState {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }

    fn will_tick(&mut self, state: &mut GameState, world: &mut World, _: &mut EventQueue) {
        //
        let state_collision = state.get_value2::<StateCollision>();
        for (_, collider) in world.query::<&mut ComponentColliderSphere>().iter() {
            let mut collision = Vec::<CollisionSnapshot>::new();

            for c in state_collision.collisions.iter() {
                let is_same = c.collider_a.guid == collider.guid;
                if is_same {
                    collision.push(c.clone());
                }
            }
            collider.collisions = collision;
        }
    }
    fn did_tick(&mut self, state: &mut GameState, world: &mut World, _: &mut EventQueue) {
        state.edit::<StateCollider>(|x| {
            for (_, (collider, transform)) in world
                .query::<(&ComponentColliderSphere, &Transform)>()
                .iter()
            {
                x.colliders
                    .push(ColliderSnapshot::new(0, transform.get_matrix(), collider.get_shape()));
            }
        });
    }
    fn debug(&mut self, state: &mut GameState, world: &mut World, _: &mut EventQueue) {
        state.edit::<GizmosState>(|x| {
            for (_, (collider, transform)) in world
                .query::<(&ComponentColliderSphere, &Transform)>()
                .iter()
            {
                x.draw_calls
                    .push(Gizmo::sphere(transform.get_matrix(), collider.diameter, Color::green()));
            }
        });
    }
}
