use crate::state::state_position_ball::StatePositionBall;
use crate::state::state_position_player::StatePositionPlayer;
use crate::state::state_turn::StateTurn;
use crate::{game_events::GameEvents, state::state_position_player};
use built_in::component::component_renderer::Renderer;
use built_in::component::component_transform::Transform;
use built_in::system::system_renderer_update_state::SystemRendererUpdateState;
use built_in_state::state_draw::DrawCallsState;
use built_in_state::state_input::InputState;
use core::collections::draw_call::DrawCall;
use core::collections::material::Material;
use core::collections::matrix4x4::Matrix4x4;
use core::collections::mesh::Mesh;
use core::collections::quaternion::Quaternion;
use core::collections::vector3::Vector3;
use core::io::asset_loader::AssetLoader;
use core::io::model_asset::ModelAsset;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};
use ecs_system::global_ecs_system;
use hecs::World;
use std::sync::Arc;

#[global_ecs_system]
pub struct ECSSystemRender {
    // mesh_tile: Option<Arc<ModelAsset>>,
}
impl ECSSystemEventless for ECSSystemRender {
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalPeer, NetworkModes::OnlinePeer]
    }
    fn is_enabled(&mut self, game_state: &mut GameState, _: &mut World) -> bool {
        game_state.get_value2::<StateTurn>().active_instance_id == game_state.instance_id
    }
    fn init(&mut self, _: &mut GameState, world: &mut World, _: &mut EventQueue, asset_loader: &mut core::io::asset_loader::AssetLoader) {
        // constant
        let tile_size = 1.0;
        let tile_spacing = 0.5;

        // let mesh = Mesh::primitive_cube(Vector3::new(tile_size, 0.1, tile_size));
        // let mat = Material::new(AssetLoader::load_shader_desc("assets/shader/my_shader.shader"));
        // let asset = Arc::new(ModelAsset::new(vec![mesh], vec![mat]));
        // for x in 0..4 {
        //     for z in 0..4 {
        //         let pos = Vector3::new((x as f32 * tile_spacing) + (x as f32 * tile_size), 0.0, (z as f32 * tile_spacing) + (z as f32 * tile_size));
        //         world.spawn((Renderer::default().set_asset(Some(asset.clone())), Transform::default().set_position(pos)));
        //     }
        // }
    }
    fn tick(&mut self, game_state: &mut GameState, _: &mut World, events: &mut EventQueue) {
        // // constant
        // let tile_size = 1.0;
        // let tile_spacing = 0.5;

        // // state

        // let state_position_player = game_state.get_value2::<StatePositionPlayer>();
        // let state_position_ball = game_state.get_value2::<StatePositionBall>();

        // let mut matrix_tile = vec![];
        // for x in 0..4 {
        //     for z in 0..4 {
        //         let pos = Vector3::new((x as f32 * tile_spacing) + (x as f32 * tile_size), 0.0, (z as f32 * tile_spacing) + (z as f32 * tile_size));
        //         matrix_tile.push(Matrix4x4::new(pos, Quaternion::identity(), Vector3::one()));
        //     }
        // }
        // // draw tiles
        // let m = self.mesh_tile.clone().unwrap();
        // game_state.edit::<DrawCallsState>(|x| {
        //     x.draw_calls
        //         .push(DrawCall::draw_mesh_instanced(m.clone(), Material::new(AssetLoader::load_shader_desc("assets/shader/my_shader.shader")), matrix_tile.clone()));
        // });
    }
}
