use ecs_event::ECSEvent;
use ecs_system::ECSSystem;
use hecs::World;

use crate::{
    dumpster_engine::EventReciever,
    gameplay::{
        ecs::component::{component_colliders::component_collider_box::ComponentColliderBox, component_transform::Transform},
        game_events::GameEvents,
    },
    my_game::ecs::component::component_ball::ComponentBall,
    random::Random,
    system::{
        system_components::gameplay_components::gameplay_component_default::{ECSSystemEventless, EventQueue, EventQueue2},
        system_game_states::state_time::TimeState,
    },
    Collections::{game_state::GameState, vector3::Vector3},
};

#[ECSSystem]
pub struct SystemBallMove {}
impl SystemBallMove {
    pub fn new() -> Box<SystemBallMove> {
        Box::new(SystemBallMove {})
    }
}
impl ECSSystemEventless for SystemBallMove {
    fn is_enabled(&mut self, game_state: &mut GameState, world: &mut World) -> bool {
        true
    }
    fn tick(&mut self, game_state: &mut GameState, world: &mut World) {
        let state_time = game_state.get_value2::<TimeState>();

        for (_, (ball, transform, collider)) in world
            .query::<(&mut ComponentBall, &mut Transform, &ComponentColliderBox)>()
            .iter()
        {
            if collider.is_colliding() {
                if Vector3::dot(ball.direction, collider.collisions[0].contact.normal_b) < 0.0 {
                    ball.direction = Vector3::reflect(ball.direction, collider.collisions[0].contact.normal_b);
                    ball.speed = ball.speed + 1.0;
                }
            }
        }
        for (_, (ball, transform)) in world.query::<(&mut ComponentBall, &mut Transform)>().iter() {
            // left - right
            if transform.position.x < -8.0 {
                ball.direction = Vector3::reflect(ball.direction, Vector3::left());
            }
            if transform.position.x > 8.0 {
                ball.direction = Vector3::reflect(ball.direction, Vector3::right());
            }

            // front - back
            if transform.position.z < -10.0 {
                ball.direction = (Random::vector3(true, false, true));
                ball.speed = 5.0;
                transform.position = Vector3::zero();
            }
            if transform.position.z > 10.0 {
                ball.speed = 5.0;
                transform.position = Vector3::zero();
                ball.direction = (Random::vector3(true, false, true));
            }

            // move
            transform.position = transform.position + ball.direction * ball.speed * state_time.scaled_delta_time;
        }
    }
}

#[ECSEvent(GameEvents)]
impl EventReciever<GameEvents> for SystemBallMove {
    fn dequeue_event(&mut self, game_state: &mut GameState, world: &mut World, event_queue: &mut EventQueue2, event: &GameEvents) {
        println!("dequeue");
        match event {
            GameEvents::A(_) => println!("A"),
            GameEvents::B(_) => println!("b"),
        }
    }
}
