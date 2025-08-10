use core::{
    collections::{color, event_queue::EventQueue2, game_state::GameState, gizmo::Gizmo},
    gameplay::ecs::{
        component::component_collider::{ColliderSnapshot, CollisionSnapshot},
        traits::ecs_system::ECSSystemEventless,
    },
    system::system_game_states::{state_colliders::StateCollider, state_collision::StateCollision, state_gizmos::GizmosState},
};
use ecs_system::global_ecs_system;
use hecs::World;

use crate::component::{component_colliders::component_collider_box::ComponentColliderBox, component_transform::Transform};

#[global_ecs_system]
pub struct SystemColliderSphereUpdateState {}
impl SystemColliderSphereUpdateState {
    pub fn new() -> Box<SystemColliderSphereUpdateState> {
        Box::new(SystemColliderSphereUpdateState {})
    }
}
impl ECSSystemEventless for SystemColliderSphereUpdateState {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn will_tick(&mut self, state: &mut GameState, world: &mut World, _: &mut EventQueue2) {
        //
        let state = state.get_value2::<StateCollision>();
        for (_, collider) in world.query::<&mut ComponentColliderBox>().iter() {
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
    fn did_tick(&mut self, state: &mut GameState, world: &mut World, _: &mut EventQueue2) {
        state.edit::<StateCollider>(|x| {
            for (_, (collider, transform)) in world.query::<(&ComponentColliderBox, &Transform)>().iter() {
                x.colliders
                    .push(ColliderSnapshot::new(collider.guid, transform.get_matrix(), collider.get_shape()));
            }
        });
    }
    fn debug(&mut self, state: &mut GameState, world: &mut World, _: &mut EventQueue2) {
        state.edit::<GizmosState>(|x| {
            for (_, (collider, transform)) in world.query::<(&ComponentColliderBox, &Transform)>().iter() {
                x.draw_calls
                    .push(Gizmo::cube(transform.get_matrix(), collider.size, color::Color::green()));
            }
        });
    }
}
