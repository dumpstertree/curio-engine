use built_in::component::{component_colliders::component_collider_box::ComponentColliderBox, component_transform::Transform};
use built_in_state::state_time::TimeState;
use ecs_event::global_ecs_system_event_reciever;
use ecs_system::global_ecs_system;
use hecs::World;

use crate::{ecs::component::component_ball::ComponentBall, game_events::GameEvents, state::state_score::StateScore};

use core::{
    collections::{
        event_queue::EventQueue,
        game_state::{self, GameState},
        vector3::Vector3,
    },
    gameplay::ecs::traits::{ecs_event_reciever::EventReciever, ecs_system::ECSSystemEventless},
    random::Random,
};

#[global_ecs_system]
pub struct SystemBallMove {}
impl ECSSystemEventless for SystemBallMove {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn tick(&mut self, state: &mut GameState, world: &mut World, _: &mut EventQueue) {
        let state_time = state.get_value2::<TimeState>();

        for (_, (ball, _, collider)) in world
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
                ball.direction = Random::direction(true, false, true);
                ball.speed = 5.0;
                transform.position = Vector3::zero();

                // update score
                state.edit::<StateScore>(|x| {
                    x.score = x.score + 1;
                });
            }
            if transform.position.z > 10.0 {
                ball.direction = Random::direction(true, false, true);
                ball.speed = 5.0;
                transform.position = Vector3::zero();

                // update score
                state.edit::<StateScore>(|x| {
                    x.score = x.score + 1;
                });
            }

            // move
            transform.position = transform.position + ball.direction * ball.speed * state_time.scaled_delta_time;

            // update score
            println!("has score : {}", state.get_value2::<StateScore>().score);
        }
    }
}

// #[global_ecs_system_event_reciever(GameEvents)]
// impl EventReciever<GameEvents> for SystemBallMove {
//     fn dequeue_event(&mut self, _: &mut GameState, _: &mut World, _: &mut EventQueue, event: &GameEvents) {
//         println!("dequeue");
//         match event {
//             GameEvents::A(_) => println!("A"),
//             GameEvents::B(_) => println!("b"),
//         }
//     }
// }
