// use crate::ecs::components::component_card::ComponentCard;
// use crate::state::peer::state_peer_selected_card::StatePeerSelectedCards;
// use crate::state::state_deck::StateDeck;
// use built_in::component::component_renderer_static::Renderer;
// // use built_in::component::component_renderer::Renderer;
// use built_in::component::component_transform::Transform;
// use built_in_state::state_camera::CameraState;
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
// use std::collections::HashMap;
// use std::sync::Arc;

// #[global_ecs_system]
// pub struct ECSSystemViewCards {
//     asset: Option<Arc<ModelAsset>>,
//     asset_card: HashMap<String, Option<Arc<ModelAsset>>>,
//     cnt: i32,
// }
// impl ECSSystemEventless for ECSSystemViewCards {
//     fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
//         vec![NetworkModes::LocalPeer, NetworkModes::OnlinePeer]
//     }
//     fn is_enabled(&mut self, game_state: &mut GameState, _: &mut World) -> bool {
//         true
//     }
//     fn init(&mut self, game_state: &mut GameState, world: &mut World, _: &mut EventQueue, asset_loader: &mut core::io::asset_loader::AssetLoader) {
//         self.asset_card
//             .insert(String::from("card_bump.glb"), AssetLoader::load_gltf("card_bump.glb"));
//         self.asset_card
//             .insert(String::from("card_set.glb"), AssetLoader::load_gltf("card_set.glb"));
//         self.asset_card
//             .insert(String::from("card_spike.glb"), AssetLoader::load_gltf("card_spike.glb"));
//     }

//     fn tick(&mut self, game_state: &mut GameState, world: &mut World, events: &mut EventQueue) {
//         self.cnt += 1;
//         if self.cnt < 15 {
//             return;
//         }
//         if self.cnt == 15 {
//             let state_deck = game_state.get_value2::<StateDeck>();
//             let my_deck = state_deck.deck.get(&game_state.instance_id).unwrap();
//             let mut i = 0;
//             for x in my_deck.hand_persistent.iter() {
//                 let camera_state = game_state.get_value2::<CameraState>();

//                 world.spawn((
//                     Renderer::default().set_asset(self.asset_card[&String::from("card_bump.glb")].clone()),
//                     Transform::default().set_rotation(camera_state.cameras.rotation),
//                     ComponentCard::default().set_index(i),
//                 ));
//                 i += 1;
//             }
//             for x in my_deck.hand_consumable.iter() {
//                 let camera_state = game_state.get_value2::<CameraState>();
//                 world.spawn((
//                     Renderer::default().set_asset(self.asset_card[&String::from("card_bump.glb")].clone()),
//                     Transform::default().set_rotation(camera_state.cameras.rotation),
//                     ComponentCard::default().set_index(i),
//                 ));
//                 i += 1;
//             }
//         }
//         let y_selected = 0.25;
//         let y_unselected = 0.5;
//         let spacing = 0.5;
//         let z_selected = 1.0;
//         let z_unselected = 1.5;

//         for (_, (card, transform)) in world.query::<(&ComponentCard, &mut Transform)>().iter() {
//             let camera_state = game_state.get_value2::<CameraState>();
//             let state_selected = game_state.get_value2::<StatePeerSelectedCards>();

//             let mut z = z_unselected;
//             let mut y = y_unselected;
//             if card.index == state_selected.index {
//                 z = z_selected;
//                 y = y_selected;
//             }

//             let pos = camera_state.cameras.position + (camera_state.cameras.rotation * Vector3::forward()) * z + Vector3::right() * ((card.index - state_selected.index) as f32 * spacing) + camera_state.cameras.rotation * Vector3::down() * y;

//             transform.position = Vector3::lerp(transform.position, pos, 0.2);
//         }
//     }
// }
