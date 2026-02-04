use crate::ecs::components::component_ball::ComponentBall;
use crate::game_board::GameBoard;
use crate::state::state_position_ball::StatePositionBall;
use curio_core::Vector3;
use curio_core::built_in::record::sys_record_time::SysRecordTime;
use curio_core::{
    collections::network_modes::NetworkModes,
    collections::{event_queue::EventQueue, game_state::GameState},
};
use gameplay::built_in::facet::renderer::renderer_static::RendererStatic;
use gameplay::built_in::facet::transform::transform3d::Transform3D;
use gameplay::context_3d::Context3D;
use gameplay::traits::habit::Habit;
use gameplay::traits::scope::Scope;
use gameplay::traits_internal::world_context_common::ContextCommon;
use habit::habit;

#[habit]
pub struct Instance {}
impl Scope for Instance {
    fn is_enabled(&mut self, _game_state: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _game_state: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all_peer()
    }
}
impl Habit for Instance {
    fn tick(&mut self, game_state: &mut GameState, world: &mut Context3D, _events: &mut EventQueue) {
        let state_position_ball = game_state.get::<StatePositionBall>();
        let state_time = game_state.get::<SysRecordTime>();

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
