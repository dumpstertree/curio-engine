use built_in::component::{component_input_index::InputIndex, component_transform::Transform};
use built_in_state::{state_input::InputState, state_time::TimeState};
use ecs_system::global_ecs_system;
use hecs::World;

use crate::{constants::Constants, ecs::component::component_paddle::ComponentPaddle};

use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};

// use dumpster_engine::
#[global_ecs_system]
pub struct SystemPaddleMove {}

impl ECSSystemEventless for SystemPaddleMove {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn tick(&mut self, state: &mut GameState, world: &mut World, _: &mut EventQueue) {
        let state_input = state.get_value2::<InputState>();
        let state_time = state.get_value2::<TimeState>();

        for (_, (input_index, paddle, transform)) in world
            .query::<(&mut InputIndex, &mut ComponentPaddle, &mut Transform)>()
            .iter()
        {
            if state_input.mapped[input_index.index]
                .get_button_or_default("move_left")
                .is_down
            {
                paddle.speed = paddle.speed + Constants::paddle_speed_acceleration() * state_time.scaled_delta_time;
            } else if state_input.mapped[input_index.index]
                .get_button_or_default("move_right")
                .is_down
            {
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

            transform.position = (transform.position - paddle.axis * paddle.speed * state_time.scaled_delta_time).clamp_x_and_copy(-10.0, 10.0);
        }
    }
}
