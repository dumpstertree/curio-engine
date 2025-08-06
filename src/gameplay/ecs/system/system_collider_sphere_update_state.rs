use crate::{
    gameplay::ecs::{
        component::{
            component_collider::{ColliderSnapshot, CollisionSnapshot},
            component_colliders::component_collider_sphere::ComponentColliderSphere,
            component_transform::Transform,
        },
        traits::ecs_system::ECSSystemEventless,
    },
    system::system_game_states::{state_colliders::StateCollider, state_collision::StateCollision, state_gizmos::GizmosState},
    Collections::{event_queue::EventQueue2, game_state::GameState, gizmo::Gizmo, Color::Color},
};
use ecs_system::ECSSystem;
use hecs::World;

#[ECSSystem]
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

    fn will_tick(&mut self, state: &mut GameState, world: &mut World, events: &mut EventQueue2) {
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
    fn did_tick(&mut self, state: &mut GameState, world: &mut World, events: &mut EventQueue2) {
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
    fn debug(&mut self, state: &mut GameState, world: &mut World, events: &mut EventQueue2) {
        state.edit::<GizmosState>(|x| {
            for (_, (collider, transform)) in world
                .query::<(&ComponentColliderSphere, &Transform)>()
                .iter()
            {
                x.draw_calls
                    .push(Gizmo::sphere(transform.get_matrix(), collider.diameter, Color::get_green()));
            }
        });
    }
}
