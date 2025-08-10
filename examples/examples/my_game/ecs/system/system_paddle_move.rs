use built_in::component::component_transform::Transform;
use ecs_system::global_ecs_system;
use hecs::World;

use crate::{constants::Constants, ecs::component::component_paddle::ComponentPaddle};

use core::{
    Collections::{event_queue::EventQueue2, game_state::GameState},
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
    system::system_game_states::{state_input::InputState, state_time::TimeState},
};

// use dumpster_engine::
#[global_ecs_system]
pub struct SystemPaddleMove {}

impl ECSSystemEventless for SystemPaddleMove {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn tick(&mut self, state: &mut GameState, world: &mut World, _: &mut EventQueue2) {
        let state_input = state.get_value2::<InputState>();
        let state_time = state.get_value2::<TimeState>();

        for (_, (paddle, _)) in world
            .query::<(&mut ComponentPaddle, &mut Transform)>()
            .iter()
        {
            if state_input.a.is_down {
                paddle.speed = paddle.speed + Constants::paddle_speed_acceleration() * state_time.scaled_delta_time;
            } else if state_input.d.is_down {
                paddle.speed = paddle.speed - Constants::paddle_speed_acceleration() * state_time.scaled_delta_time;
            } else {
                if f32::abs(paddle.speed) < 0.5 {
                    paddle.speed = 0.0;
                } else if paddle.speed > 0.0 {
                    paddle.speed = paddle.speed - Constants::paddle_speed_decceleration() * state_time.scaled_delta_time;
                } else {
                    paddle.speed = paddle.speed + Constants::paddle_speed_decceleration() * state_time.scaled_delta_time;
                }
            }

            paddle.speed = paddle
                .speed
                .clamp(-Constants::paddle_speed_terminal(), Constants::paddle_speed_terminal());

            // transform.position = (transform.position - paddle.axis * paddle.speed * state_time.scaled_delta_time).clamp_x(-10.0, 10.0);
        }
    }
}
