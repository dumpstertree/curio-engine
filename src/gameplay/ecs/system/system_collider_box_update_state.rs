use crate::{
    dumpster_engine::EventReciever,
    gameplay::{
        ecs::component::{
            component_collider::{ColliderSnapshot, CollisionSnapshot},
            component_colliders::component_collider_box::ComponentColliderBox,
            component_transform::Transform,
        },
        game_events::GameEvents,
    },
    system::{
        system_components::gameplay_components::gameplay_component_default::{EngineCommands, EventQueue, EventQueue2},
        system_game_states::{state_colliders::StateCollider, state_collision::StateCollision, state_gizmos::GizmosState},
    },
    Collections::{game_state::GameState, gizmo::Gizmo, vector3::Vector3, Color},
};
use ecs_event::ECSEvent;
use ecs_system::ECSSystem;
use hecs::World;
use intertrait::cast_to;

use crate::system::system_components::gameplay_components::gameplay_component_default::ECSSystemEventless;

#[ECSSystem]
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
    fn will_tick(&mut self, game_state: &mut GameState, world: &mut World, event_queue: &mut EventQueue2) {
        // test
        event_queue.enqueue_event(GameEvents::A("AHHHH".to_string()));
        //
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
        game_state.edit::<StateCollider>(|x| {
            for (_, (collider, transform)) in world.query::<(&ComponentColliderBox, &Transform)>().iter() {
                x.colliders
                    .push(ColliderSnapshot::new(collider.guid, transform.get_matrix(), collider.get_shape()));
            }
        });
    }
    fn debug(&mut self, game_state: &mut GameState, world: &mut World, system_event_queue: &mut EventQueue<EngineCommands>) {
        game_state.edit::<GizmosState>(|x| {
            for (_, (collider, transform)) in world.query::<(&ComponentColliderBox, &Transform)>().iter() {
                x.draw_calls
                    .push(Gizmo::cube(transform.get_matrix(), collider.size, Color::Color::get_green()));
            }
        });
    }
}
#[ECSEvent(GameEvents)]
impl EventReciever<GameEvents> for SystemColliderSphereUpdateState {
    fn dequeue_event(&mut self, game_state: &mut GameState, world: &mut World, event_queue: &mut EventQueue2, event: &GameEvents) {
        println!("found! other");
    }
}
