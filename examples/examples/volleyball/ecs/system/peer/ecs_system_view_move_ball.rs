use crate::AssetMappingUIDs;
use crate::ecs::components::component_ball::ComponentBall;
use crate::game_board::GameBoard;
use crate::state::state_position_ball::StatePositionBall;
use built_in::component::component_renderer_animated::RendererAnimated;
use built_in::component::component_renderer_static::Renderer;
use built_in::component::component_transform::Transform;
use built_in_state::state_time::TimeState;
use core::collections::quaternion::Quaternion;
use core::collections::vector3::Vector3;
use core::io::asset_loader::AssetLoader;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};
use ecs_system::global_ecs_system;
use hecs::World;

#[global_ecs_system]
pub struct ECSSystemViewMoveBall {}
impl ECSSystemEventless for ECSSystemViewMoveBall {
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalPeer, NetworkModes::OnlinePeer]
    }
    fn is_enabled(&mut self, game_state: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn enable(&mut self, _: &mut GameState, world: &mut World, _: &mut EventQueue) {
        let mut r = Renderer::default();
        r = r.set_asset(Some(AssetLoader::load_model_static_from_database(AssetMappingUIDs::Ball.uid())));
        world.spawn((Transform::default(), r, ComponentBall::default()));
    }
    fn tick(&mut self, game_state: &mut GameState, world: &mut World, events: &mut EventQueue) {
        let state_position_ball = game_state.get::<StatePositionBall>();
        let state_time = game_state.get::<TimeState>();

        for (_, (transform, _, renderer)) in world
            .query::<(&mut Transform, &ComponentBall, &mut Renderer)>()
            .iter()
        {
            let loc = (state_position_ball.column, state_position_ball.row);

            // get pos
            let tar_pos = GameBoard::get_world_position(loc.0, loc.1) + Vector3::up();

            //move towards position and get back the delta
            transform.move_towards_position(tar_pos, 10.0 * state_time.scaled_delta_time);
            transform.scale = Vector3::one() * 0.25;
        }
    }
}
