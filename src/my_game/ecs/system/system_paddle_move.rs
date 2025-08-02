use hecs::World;

use crate::{
    gameplay::{ecs::component::component_transform::Transform, game_events::GameEvents},
    my_game::{constants::Constants, ecs::component::component_paddle::ComponentPaddle},
    system::{
        system_components::gameplay_components::gameplay_component_default::{ECSSystem, EventQueue},
        system_game_states::{state_input::InputState, state_time::TimeState},
    },
    Collections::{game_state::GameState, quaternion::Quaternion, vector3::Vector3},
};

pub struct SystemPaddleMove {}
impl SystemPaddleMove {
    pub fn new() -> Box<SystemPaddleMove> {
        Box::new(SystemPaddleMove {})
    }
}

impl ECSSystem<GameEvents> for SystemPaddleMove {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World, _: &mut EventQueue<GameEvents>) -> bool {
        true
    }
    fn tick(&mut self, game_state: &mut GameState, world: &mut World, _: &mut EventQueue<GameEvents>) {
        let state_input = game_state.get_value2::<InputState>();
        let state_time = game_state.get_value2::<TimeState>();

        for (_, (paddle, transform)) in world
            .query::<(&mut ComponentPaddle, &mut Transform)>()
            .iter()
        {
            if state_input.a.is_down {
                println!("a is down");
                paddle.speed = paddle.speed + Constants::paddle_speed_acceleration() * state_time.delta_time;
            } else if state_input.d.is_down {
                paddle.speed = paddle.speed - Constants::paddle_speed_acceleration() * state_time.delta_time;
            } else {
                if f32::abs(paddle.speed) < 0.5 {
                    paddle.speed = 0.0;
                } else if paddle.speed > 0.0 {
                    paddle.speed = paddle.speed - Constants::paddle_speed_decceleration() * state_time.delta_time;
                } else {
                    paddle.speed = paddle.speed + Constants::paddle_speed_decceleration() * state_time.delta_time;
                }
            }

            paddle.speed = paddle
                .speed
                .clamp(-Constants::paddle_speed_terminal(), Constants::paddle_speed_terminal());

            transform.position = (transform.position - paddle.axis * paddle.speed * state_time.delta_time).clamp_x(-10.0, 10.0);
        }
    }
}
