use crate::{
    gameplay::ecs::component::{
        component_collider::{ColliderSnapshot, CollisionSnapshot},
        component_colliders::component_collider_box::ComponentColliderBox,
        component_transform::Transform,
    },
    system::system_game_states::{state_colliders::StateCollider, state_collision::StateCollision, state_gizmos::GizmosState, state_time::TimeState},
    Collections::{
        game_state::{self, GameState, GetError},
        gizmo::Gizmo,
        matrix4x4::Matrix4x4,
        quaternion::Quaternion,
        vector3::Vector3,
        Color,
    },
};
use hecs::World;

use crate::system::system_components::gameplay_components::gameplay_component_default::ECSSystemEventless;

pub struct SystemColliderSphereUpdateState {}
impl SystemColliderSphereUpdateState {
    pub fn new() -> Box<SystemColliderSphereUpdateState> {
        Box::new(SystemColliderSphereUpdateState {})
    }
}
impl ECSSystemEventless for SystemColliderSphereUpdateState {
    fn is_enabled(&mut self, game_state: &mut GameState, world: &mut World) -> bool {
        true
    }
    fn will_tick(&mut self, game_state: &mut GameState, world: &mut World) {
        let state = game_state.get_value2::<StateCollision>();
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
    fn did_tick(&mut self, game_state: &mut GameState, world: &mut World) {
        let mut gizmo = game_state.get_value2::<GizmosState>();

        let mut state = game_state.get_value2::<StateCollider>();
        for (_, (collider, transform)) in world.query::<(&ComponentColliderBox, &Transform)>().iter() {
            state
                .colliders
                .push(ColliderSnapshot::new(collider.guid, transform.get_matrix(), collider.get_shape()));
            gizmo
                .draw_calls
                .push(Gizmo::cube(transform.get_matrix(), Vector3::new(3.0, 1.0, 1.0), Color::Color::get_red()));
            gizmo
                .draw_calls
                .push(Gizmo::sphere(transform.get_matrix(), 7.0, Color::Color::get_green()));
        }
        game_state.set_value2::<StateCollider>(state);
        game_state.set_value2::<GizmosState>(gizmo);
    }
}
