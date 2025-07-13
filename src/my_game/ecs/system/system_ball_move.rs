use hecs::World;

use crate::{
    gameplay::{ecs::component::component_transform::Transform, game_events::GameEvents},
    my_game::ecs::component::component_ball::ComponentBall,
    system::{
        system_components::gameplay_components::gameplay_component_default::{ECSSystem, EventQueue},
        system_game_states::state_time::TimeState,
    },
    Collections::{game_state::GameState, vector3::Vector3},
};

pub struct SystemBallMove {}
impl SystemBallMove {
    pub fn new() -> Box<SystemBallMove> {
        Box::new(SystemBallMove {})
    }
}
impl ECSSystem<GameEvents> for SystemBallMove {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World, _: &mut EventQueue<GameEvents>) -> bool {
        true
    }
    fn tick(&mut self, game_state: &mut GameState, world: &mut World, _: &mut EventQueue<GameEvents>) {
        let state_time = game_state.get_value2::<TimeState>();

        for (_, (ball, transform)) in world.query::<(&mut ComponentBall, &mut Transform)>().iter() {
            // left - right
            if transform.position.x < -10.0 {
                ball.direction = Vector3::reflect(ball.direction, Vector3::left());
            }
            if transform.position.x > 10.0 {
                ball.direction = Vector3::reflect(ball.direction, Vector3::right());
            }

            // front - back
            if transform.position.z < -10.0 {
                ball.direction = Vector3::reflect(ball.direction, Vector3::forward());
            }
            if transform.position.z > 10.0 {
                ball.direction = Vector3::reflect(ball.direction, Vector3::back());
            }

            // move
            transform.position = transform.position + ball.direction * ball.speed * state_time.delta_time;
        }
    }
}
