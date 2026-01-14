use crate::ecs::components::component_ball::ComponentBall;
use crate::game_board::GameBoard;
use crate::state::state_position_ball::StatePositionBall;
use built_in_state::state_time::TimeState;
use curio_core::collections::vector3::Vector3;
use curio_core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_system::habit;
use system_component_default_gameplay::built_in::facet::renderer::renderer_static::RendererStatic;
use system_component_default_gameplay::built_in::facet::transform::transform3d::Transform3D;
use system_component_default_gameplay::context_3d::Context3D;
use system_component_default_gameplay::traits::habit::Habit;
use system_component_default_gameplay::traits::scope::Scope;
use system_component_default_gameplay::traits_internal::world_context_common::ContextCommon;

#[habit]
pub struct Instance {}
impl Scope for Instance {
    fn is_enabled(&mut self, game_state: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all_peer()
    }
}
impl Habit for Instance {
    fn tick(&mut self, game_state: &mut GameState, world: &mut Context3D, _events: &mut EventQueue) {
        let state_position_ball = game_state.get::<StatePositionBall>();
        let state_time = game_state.get::<TimeState>();

        world.edit::<(&mut Transform3D, &ComponentBall, &mut RendererStatic)>(|q| {
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
