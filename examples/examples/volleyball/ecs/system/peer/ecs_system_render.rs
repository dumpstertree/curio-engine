// use crate::ecs::components::component_ball::ComponentBall;
// use crate::ecs::components::component_player::ComponentPlayer;
// use crate::game_board::GameBoard;
// use crate::game_events;
// use crate::state::state_position_ball::StatePositionBall;
// use crate::state::state_position_player::StatePositionPlayer;
// use crate::state::state_teams::Teams;
// use crate::state::state_turn::StateTurn;
// use crate::{game_events::GameEvents, state::state_position_player};
// use built_in::component::component_renderer::Renderer;
// use built_in::component::component_transform::Transform;
// use built_in::system::system_renderer_update_state::SystemRendererUpdateState;
// use built_in_state::state_camera::CameraState;
// use built_in_state::state_draw::DrawCallsState;
// use built_in_state::state_input::InputState;
// use core::collections::camera_uniform::CameraSnapshot;
// use core::collections::draw_call::DrawCall;
// use core::collections::game_state;
// use core::collections::material::Material;
// use core::collections::matrix4x4::Matrix4x4;
// use core::collections::mesh::Mesh;
// use core::collections::quaternion::Quaternion;
// use core::collections::vector3::Vector3;
// use core::io::asset_loader::AssetLoader;
// use core::io::model_asset::ModelAsset;
// use core::{
//     collections::{event_queue::EventQueue, game_state::GameState},
//     dumpster_engine::NetworkModes,
//     gameplay::ecs::traits::ecs_system::ECSSystemEventless,
// };
// use ecs_system::global_ecs_system;
// use hecs::World;
// use std::sync::Arc;

// #[global_ecs_system]
// pub struct ECSSystemRender {
//     // mesh_tile: Option<Arc<ModelAsset>>,
//     asset: Option<Arc<ModelAsset>>,

//     asset_ball: Option<Arc<ModelAsset>>,
//     cnt: i32,
// }
// impl ECSSystemEventless for ECSSystemRender {
//     fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
//         vec![NetworkModes::LocalPeer, NetworkModes::OnlinePeer]
//     }
//     fn is_enabled(&mut self, game_state: &mut GameState, _: &mut World) -> bool {
//         true
//     }
//     fn init(&mut self, game_state: &mut GameState, world: &mut World, _: &mut EventQueue, asset_loader: &mut core::io::asset_loader::AssetLoader) {
//         let asset = asset_loader.load_gltf("tile.glb").unwrap();
//         for x in 0..4 {
//             for z in 0..4 {
//                 let pos = GameBoard::get_world_position(x, z);
//                 world.spawn((Renderer::default().set_asset(Some(asset.clone())), Transform::default().set_position(pos)));
//             }
//         }
//         self.asset = asset_loader.load_gltf("player.glb");
//         self.asset_ball = asset_loader.load_gltf("ball.glb");
//     }
//     fn enable(&mut self, game_state: &mut GameState, world: &mut World, _: &mut EventQueue) {}
//     fn tick(&mut self, game_state: &mut GameState, world: &mut World, events: &mut EventQueue) {
//         self.cnt += 1;
//         if self.cnt < 10 {
//             return;
//         }
//         if self.cnt == 10 {
//             for x in game_state.get_value2::<StatePositionPlayer>().positions {
//                 let pos = GameBoard::get_world_position(x.1.0, x.1.1);
//                 world.spawn((ComponentPlayer::default().set_player_id(x.0), Renderer::default().set_asset(self.asset.clone()), Transform::default().set_position(pos)));
//             }
//             world.spawn((ComponentBall::default(), Renderer::default().set_asset(self.asset_ball.clone()), Transform::default()));
//         }
//         let state_player_positions = game_state.get_value2::<StatePositionPlayer>();
//         for (_, (player, transform)) in world.query::<(&ComponentPlayer, &mut Transform)>().iter() {
//             let loc = state_player_positions
//                 .positions
//                 .get(&player.player_id)
//                 .unwrap();
//             transform.position = Vector3::lerp(transform.position, GameBoard::get_world_position(loc.0, loc.1), 0.2);
//         }
//         let state_ball_positions = game_state.get_value2::<StatePositionBall>();
//         for (_, (ball, transform)) in world.query::<(&ComponentBall, &mut Transform)>().iter() {
//             transform.position = Vector3::lerp(transform.position, GameBoard::get_world_position(state_ball_positions.collun, state_ball_positions.row), 0.1);
//         }
//     }
// }
