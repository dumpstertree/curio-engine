use crate::ecs::components::component_ball::ComponentBall;
use crate::game_board::GameBoard;
use crate::state::state_position_ball::StatePositionBall;
use built_in_state::state_time::TimeState;
use core::collections::vector3::Vector3;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_system::global_ecs_system;
use system_component_default_gameplay::component::component_renderer_static::Renderer;
use system_component_default_gameplay::component::component_transform::Transform;
use system_component_default_gameplay::traits::ecs_system::ECSSystemEventless;
use system_component_default_gameplay::world_context::{WorldContext, WorldContextCommon};

#[global_ecs_system]
pub struct ECSSystemViewMoveBall {}
impl ECSSystemEventless for ECSSystemViewMoveBall {
    fn run_on_instance(&mut self, _game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalPeer, NetworkModes::OnlinePeer]
    }
    fn is_enabled(&mut self, _game_state: &mut GameState, _: &mut WorldContext) -> bool {
        true
    }
    fn tick(&mut self, game_state: &mut GameState, world: &mut WorldContext, _events: &mut EventQueue) {
        let state_position_ball = game_state.get::<StatePositionBall>();
        let state_time = game_state.get::<TimeState>();

        world.query_mut::<(&mut Transform, &ComponentBall, &mut Renderer)>(|q| {
            for (_, (transform, _ball, _renderer)) in q {
                let loc = (state_position_ball.column, state_position_ball.row);

                // get pos
                let tar_pos = GameBoard::get_world_position(loc.0, loc.1) + Vector3::up();

                //move towards position and get back the delta
                transform.move_towards_position(tar_pos, 20.0 * state_time.scaled_delta_time);
                transform.scale = Vector3::one() * 0.25;
            }
        });
    }
}
