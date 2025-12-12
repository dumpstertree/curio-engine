// use built_in::component::{
//     component_renderer_animated::RendererAnimated,
//     component_renderer_static::Renderer,
//     component_renderer_text::{ComponentRendererText, RendererCommon},
//     component_transform::Transform,
// };
// use built_in_state::{state_camera::CameraState, state_input::InputState, state_time::TimeState};
// use core::{
//     collections::{color::Color, event_queue::EventQueue, game_state::GameState, input_button::InputButtonState, quaternion::Quaternion, vector2::Vector2, vector3::Vector3},
//     dumpster_engine::NetworkModes,
//     gameplay::{
//         ecs::traits::{
//             ecs_event_reciever::{self, InstanceLimiter},
//             ecs_system::ECSSystemEventless,
//         },
//         world_context::WorldContext,
//     },
//     io::asset_loader::AssetLoader,
// };
// use ecs_event::global_ecs_system_event_reciever;
// use ecs_system::global_ecs_system;
// use hecs::{Entity, World};
// use serde::{Deserialize, Serialize};
// use std::{string, sync::Arc};

// use crate::{
//     AssetMappingUIDs,
//     cards::card_instance::CardInstance,
//     ecs::components::{component_card::ComponentCard, component_energy_token::ComponentEnergyToken, component_player::ComponentPlayer, component_ui_ball_state::ComponentUIBallState, component_ui_score::ComponentUIScoreState, component_ui_turn::ComponentUITurnState},
//     exploration::exploration_path::RoomTypes,
//     game_events::GameEvents,
//     state::{
//         host::{state_currency::StateCurrency, state_exploration::StateExploration},
//         peer::state_peer_entity_ids::{EntityIDTypes, StateEntityIDs},
//         state_deck::{Deck, StateDeck},
//         state_teams::{StateTeamAssignments, Teams},
//     },
// };

// #[global_ecs_system]
// #[global_ecs_system_event_reciever(GameEvents)]
// pub struct Listener {}

// impl ECSSystemEventless for Listener {
//     fn is_enabled(&mut self, game_state: &mut GameState, world: &mut WorldContext) -> bool {
//         game_state
//             .get::<StateExploration>()
//             .exploration
//             .get_cur_room()
//             .room_type
//             == RoomTypes::Heal
//     }
//     fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<NetworkModes> {
//         NetworkModes::all_peer()
//     }
//     fn tick(&mut self, game_state: &mut GameState, world: &mut WorldContext, event_queue: &mut EventQueue) {
//         let state_camera = game_state.get::<CameraState>();
//         unsafe {
//             if game_state.get::<InputState>().mapped[0]
//                 .get_button_or_default("turn_end")
//                 .went_down
//             {
//                 if INDEX_SELECTED == 0 {
//                     event_queue.enqueue_event(GameEvents::RequestHeal(game_state.instance_id));
//                     return;
//                 }
//                 if INDEX_SELECTED == 1 {
//                     event_queue.enqueue_event(GameEvents::RequestLeaveExplorationRoom);
//                     return;
//                 }
//             }
//             if game_state.get::<InputState>().mapped[0]
//                 .get_button_or_default("move_forward")
//                 .went_up
//             {
//                 INDEX_SELECTED += 1;
//                 if INDEX_SELECTED > 1 {
//                     INDEX_SELECTED = 0;
//                 }
//             }
//             if game_state.get::<InputState>().mapped[0]
//                 .get_button_or_default("move_back")
//                 .went_up
//             {
//                 INDEX_SELECTED -= 1;
//                 if INDEX_SELECTED < 0 {
//                     INDEX_SELECTED = 1;
//                 }
//             }

//             //
//             if let Some(e) = ENTITY_DESC {
//                 let x = world.get::<&mut Transform>(e);
//                 if let Ok(mut transform) = x {
//                     use built_in::component::{
//                         component_renderer_animated::RendererAnimated,
//                         component_renderer_static::Renderer,
//                         component_renderer_text::{ComponentRendererText, RendererCommon},
//                         component_transform::Transform,
//                     };
//                     use built_in_state::{state_camera::CameraState, state_input::InputState, state_time::TimeState};
//                     use core::{
//                         collections::{color::Color, event_queue::EventQueue, game_state::GameState, input_button::InputButtonState, quaternion::Quaternion, vector2::Vector2, vector3::Vector3},
//                         dumpster_engine::NetworkModes,
//                         gameplay::{
//                             ecs::traits::{
//                                 ecs_event_reciever::{self, InstanceLimiter},
//                                 ecs_system::ECSSystemEventless,
//                             },
//                             world_context::WorldContext,
//                         },
//                         io::asset_loader::AssetLoader,
//                     };
//                     use ecs_event::global_ecs_system_event_reciever;
//                     use ecs_system::global_ecs_system;
//                     use hecs::{Entity, World};
//                     use serde::{Deserialize, Serialize};
//                     use std::{string, sync::Arc};

//                     use crate::{
//                         AssetMappingUIDs,
//                         cards::card_instance::CardInstance,
//                         ecs::components::{component_card::ComponentCard, component_energy_token::ComponentEnergyToken, component_player::ComponentPlayer, component_ui_ball_state::ComponentUIBallState, component_ui_score::ComponentUIScoreState, component_ui_turn::ComponentUITurnState},
//                         exploration::exploration_path::RoomTypes,
//                         game_events::GameEvents,
//                         state::{
//                             host::{state_currency::StateCurrency, state_exploration::StateExploration},
//                             peer::state_peer_entity_ids::{EntityIDTypes, StateEntityIDs},
//                             state_deck::{Deck, StateDeck},
//                             state_teams::{StateTeamAssignments, Teams},
//                         },
//                     };

//                     #[global_ecs_system]
//                     #[global_ecs_system_event_reciever(GameEvents)]
//                     pub struct Listener {}

//                     impl ECSSystemEventless for Listener {
//                         fn is_enabled(&mut self, game_state: &mut GameState, world: &mut WorldContext) -> bool {
//                             game_state
//                                 .get::<StateExploration>()
//                                 .exploration
//                                 .get_cur_room()
//                                 .room_type
//                                 == RoomTypes::Heal
//                         }
//                         fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<NetworkModes> {
//                             NetworkModes::all_peer()
//                         }
//                         fn tick(&mut self, game_state: &mut GameState, world: &mut WorldContext, event_queue: &mut EventQueue) {
//                             let state_camera = game_state.get::<CameraState>();
//                             unsafe {
//                                 if game_state.get::<InputState>().mapped[0]
//                                     .get_button_or_default("turn_end")
//                                     .went_down
//                                 {
//                                     if INDEX_SELECTED == 0 {
//                                         event_queue.enqueue_event(GameEvents::RequestHeal(game_state.instance_id));
//                                         return;
//                                     }
//                                     if INDEX_SELECTED == 1 {
//                                         event_queue.enqueue_event(GameEvents::RequestLeaveExplorationRoom);
//                                         return;
//                                     }
//                                 }
//                                 if game_state.get::<InputState>().mapped[0]
//                                     .get_button_or_default("move_forward")
//                                     .went_up
//                                 {
//                                     INDEX_SELECTED += 1;
//                                     if INDEX_SELECTED > 1 {
//                                         INDEX_SELECTED = 0;
//                                     }
//                                 }
//                                 if game_state.get::<InputState>().mapped[0]
//                                     .get_button_or_default("move_back")
//                                     .went_up
//                                 {
//                                     INDEX_SELECTED -= 1;
//                                     if INDEX_SELECTED < 0 {
//                                         INDEX_SELECTED = 1;
//                                     }
//                                 }

//                                 //
//                                 if let Some(e) = ENTITY_DESC {
//                                     let x = world.get::<&mut Transform>(e);
//                                     if let Ok(mut transform) = x {
//                                         let z = 1.0;
//                                         let x = 0.0;
//                                         let y = 0.0;
//                                         transform.scale = Vector3::one() * 1.0;
//                                         transform.rotation = game_state.get::<CameraState>().cameras.rotation * Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0));
//                                         transform.position = state_camera.cameras.position + (state_camera.cameras.rotation * Vector3::forward()) * z + state_camera.cameras.rotation * Vector3::down() * y + state_camera.cameras.rotation * Vector3::right() * x;
//                                     }
//                                     let x = world.get::<&mut ComponentRendererText>(e);
//                                     if let Ok(mut rend) = x {
//                                         rend.set_contents(&format!("Heal? You have {} of {}", 0, 10));
//                                         rend.set_tint(Color::red());
//                                     }
//                                 }
//                                 if let Some(e) = ENTITY_OPT_0 {
//                                     let x = world.get::<&mut Transform>(e);
//                                     if let Ok(mut transform) = x {
//                                         let z = 1.0;
//                                         let x = 0.0;
//                                         let y = 0.2;
//                                         if INDEX_SELECTED == 0 {
//                                             let sin = f64::sin(game_state.get::<TimeState>().unscaled_time * 10.0) as f32;
//                                             transform.scale = Vector3::one() * (0.5 + 0.05 * sin);
//                                         } else {
//                                             transform.scale = Vector3::one() * 0.5;
//                                         }
//                                         transform.rotation = game_state.get::<CameraState>().cameras.rotation * Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0));
//                                         transform.position = state_camera.cameras.position + (state_camera.cameras.rotation * Vector3::forward()) * z + state_camera.cameras.rotation * Vector3::down() * y + state_camera.cameras.rotation * Vector3::right() * x;
//                                     }
//                                     let x = world.get::<&mut ComponentRendererText>(e);
//                                     if let Ok(mut rend) = x {
//                                         let state_currency = game_state.get::<StateCurrency>();
//                                         if state_currency.currency >= 100 {
//                                             rend.set_contents(&format!("Heal +1 for 100g"));
//                                             rend.set_tint(Color::red());
//                                         } else {
//                                             rend.set_contents(&format!("Notenough money.Need 100 have {}", state_currency.currency));
//                                             rend.set_tint(Color::red());
//                                         }
//                                     }
//                                 }
//                                 if let Some(e) = ENTITY_OPT_1 {
//                                     let x = world.get::<&mut Transform>(e);
//                                     if let Ok(mut transform) = x {
//                                         let z = 1.0;
//                                         let x = 0.0;
//                                         let y = 0.3;
//                                         if INDEX_SELECTED == 1 {
//                                             let sin = f64::sin(game_state.get::<TimeState>().unscaled_time * 10.0) as f32;
//                                             transform.scale = Vector3::one() * (0.5 + 0.05 * sin);
//                                         } else {
//                                             transform.scale = Vector3::one() * 0.5;
//                                         }
//                                         transform.rotation = game_state.get::<CameraState>().cameras.rotation * Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0));
//                                         transform.position = state_camera.cameras.position + (state_camera.cameras.rotation * Vector3::forward()) * z + state_camera.cameras.rotation * Vector3::down() * y + state_camera.cameras.rotation * Vector3::right() * x;
//                                     }
//                                     let x = world.get::<&mut ComponentRendererText>(e);
//                                     if let Ok(mut rend) = x {
//                                         rend.set_contents(&format!("Leave"));
//                                         rend.set_tint(Color::red());
//                                     }
//                                 }
//                             }
//                         }
//                     }

//                     // Impl - Instance
//                     impl InstanceLimiter for Listener {
//                         fn is_enabled(&mut self, _game_state: &mut GameState) -> bool {
//                             true
//                         }
//                         fn run_on_instance(&mut self, _game_state: &mut GameState) -> Vec<NetworkModes> {
//                             NetworkModes::all_peer()
//                         }
//                     }
//                     // Impl - Listener
//                     impl ecs_event_reciever::EventReciever<GameEvents> for Listener {
//                         fn dequeue_event(&mut self, game_state: &mut GameState, world: &mut WorldContext, event_queue: &mut EventQueue, event: &GameEvents) {
//                             match event {
//                                 GameEvents::EnableUIHealing => {
//                                     println!("enable ui healing");
//                                     Self::spawn_dialog(game_state, world);
//                                     Self::spawn_selection(game_state, world);
//                                 }
//                                 _ => {}
//                             }
//                         }
//                     }
//                     impl Listener {
//                         fn spawn_dialog(game_state: &mut GameState, world: &mut World) {
//                             unsafe {
//                                 println!("enable ui");
//                                 let e = world.spawn((Transform::default(), ComponentRendererText::default()));
//                                 game_state.edit::<StateEntityIDs>(|x| {
//                                     x.add(EntityIDTypes::UIPanelHealing, e);
//                                 });
//                                 ENTITY_DESC = Some(e);
//                             }
//                         }
//                         fn spawn_selection(game_state: &mut GameState, world: &mut World) {
//                             unsafe {
//                                 let action_heal = world.spawn((Transform::default(), ComponentRendererText::default()));
//                                 game_state.edit::<StateEntityIDs>(|x| {
//                                     x.add(EntityIDTypes::UIPanelHealing, action_heal);
//                                 });
//                                 let action_cancel = world.spawn((Transform::default(), ComponentRendererText::default()));
//                                 game_state.edit::<StateEntityIDs>(|x| {
//                                     x.add(EntityIDTypes::UIPanelHealing, action_cancel);
//                                 });

//                                 ENTITY_OPT_0 = Some(action_heal);
//                                 ENTITY_OPT_1 = Some(action_cancel);
//                             }
//                         }
//                     }

//                     static mut ENTITY_DESC: Option<Entity> = None;
//                     static mut ENTITY_OPT_0: Option<Entity> = None;
//                     static mut ENTITY_OPT_1: Option<Entity> = None;
//                     static mut INDEX_SELECTED: i32 = 0;

//                     let z = 1.0;
//                     let x = 0.0;
//                     let y = 0.0;
//                     transform.scale = Vector3::one() * 1.0;
//                     transform.rotation = game_state.get::<CameraState>().cameras.rotation * Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0));
//                     transform.position = state_camera.cameras.position + (state_camera.cameras.rotation * Vector3::forward()) * z + state_camera.cameras.rotation * Vector3::down() * y + state_camera.cameras.rotation * Vector3::right() * x;
//                 }
//                 let x = world.get::<&mut ComponentRendererText>(e);
//                 if let Ok(mut rend) = x {
//                     rend.set_contents(&format!("Heal? You have {} of {}", 0, 10));
//                     rend.set_tint(Color::red());
//                 }
//             }
//             if let Some(e) = ENTITY_OPT_0 {
//                 let x = world.get::<&mut Transform>(e);
//                 if let Ok(mut transform) = x {
//                     let z = 1.0;
//                     let x = 0.0;
//                     let y = 0.2;
//                     if INDEX_SELECTED == 0 {
//                         let sin = f64::sin(game_state.get::<TimeState>().unscaled_time * 10.0) as f32;
//                         transform.scale = Vector3::one() * (0.5 + 0.05 * sin);
//                     } else {
//                         transform.scale = Vector3::one() * 0.5;
//                     }
//                     transform.rotation = game_state.get::<CameraState>().cameras.rotation * Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0));
//                     transform.position = state_camera.cameras.position + (state_camera.cameras.rotation * Vector3::forward()) * z + state_camera.cameras.rotation * Vector3::down() * y + state_camera.cameras.rotation * Vector3::right() * x;
//                 }
//                 let x = world.get::<&mut ComponentRendererText>(e);
//                 if let Ok(mut rend) = x {
//                     let state_currency = game_state.get::<StateCurrency>();
//                     if state_currency.currency >= 100 {
//                         rend.set_contents(&format!("Heal +1 for 100g"));
//                         rend.set_tint(Color::red());
//                     } else {
//                         rend.set_contents(&format!("Notenough money.Need 100 have {}", state_currency.currency));
//                         rend.set_tint(Color::red());
//                     }
//                 }
//             }
//             if let Some(e) = ENTITY_OPT_1 {
//                 let x = world.get::<&mut Transform>(e);
//                 if let Ok(mut transform) = x {
//                     let z = 1.0;
//                     let x = 0.0;
//                     let y = 0.3;
//                     if INDEX_SELECTED == 1 {
//                         let sin = f64::sin(game_state.get::<TimeState>().unscaled_time * 10.0) as f32;
//                         transform.scale = Vector3::one() * (0.5 + 0.05 * sin);
//                     } else {
//                         transform.scale = Vector3::one() * 0.5;
//                     }
//                     transform.rotation = game_state.get::<CameraState>().cameras.rotation * Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0));
//                     transform.position = state_camera.cameras.position + (state_camera.cameras.rotation * Vector3::forward()) * z + state_camera.cameras.rotation * Vector3::down() * y + state_camera.cameras.rotation * Vector3::right() * x;
//                 }
//                 let x = world.get::<&mut ComponentRendererText>(e);
//                 if let Ok(mut rend) = x {
//                     rend.set_contents(&format!("Leave"));
//                     rend.set_tint(Color::red());
//                 }
//             }
//         }
//     }
// }

// // Impl - Instance
// impl InstanceLimiter for Listener {
//     fn is_enabled(&mut self, _game_state: &mut GameState) -> bool {
//         true
//     }
//     fn run_on_instance(&mut self, _game_state: &mut GameState) -> Vec<NetworkModes> {
//         NetworkModes::all_peer()
//     }
// }
// // Impl - Listener
// impl ecs_event_reciever::EventReciever<GameEvents> for Listener {
//     fn dequeue_event(&mut self, game_state: &mut GameState, world: &mut WorldContext, event_queue: &mut EventQueue, event: &GameEvents) {
//         match event {
//             GameEvents::EnableUIHealing => {
//                 println!("enable ui healing");
//                 Self::spawn_dialog(game_state, world);
//                 Self::spawn_selection(game_state, world);
//             }
//             _ => {}
//         }
//     }
// }
// impl Listener {
//     fn spawn_dialog(game_state: &mut GameState, world: &mut World) {
//         unsafe {
//             println!("enable ui");
//             let e = world.spawn((Transform::default(), ComponentRendererText::default()));
//             game_state.edit::<StateEntityIDs>(|x| {
//                 x.add(EntityIDTypes::UIPanelHealing, e);
//             });
//             ENTITY_DESC = Some(e);
//         }
//     }
//     fn spawn_selection(game_state: &mut GameState, world: &mut World) {
//         unsafe {
//             let action_heal = world.spawn((Transform::default(), ComponentRendererText::default()));
//             game_state.edit::<StateEntityIDs>(|x| {
//                 x.add(EntityIDTypes::UIPanelHealing, action_heal);
//             });
//             let action_cancel = world.spawn((Transform::default(), ComponentRendererText::default()));
//             game_state.edit::<StateEntityIDs>(|x| {
//                 x.add(EntityIDTypes::UIPanelHealing, action_cancel);
//             });

//             ENTITY_OPT_0 = Some(action_heal);
//             ENTITY_OPT_1 = Some(action_cancel);
//         }
//     }
// }

// static mut ENTITY_DESC: Option<Entity> = None;
// static mut ENTITY_OPT_0: Option<Entity> = None;
// static mut ENTITY_OPT_1: Option<Entity> = None;
// static mut INDEX_SELECTED: i32 = 0;
