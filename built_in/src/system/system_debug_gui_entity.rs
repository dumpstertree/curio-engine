// use core::{
//     collections::{color, event_queue::EventQueue, game_state::GameState, gizmo::Gizmo, matrix4x4::Matrix4x4, vector3::Vector3},
//     gameplay::ecs::traits::ecs_system::ECSSystemEventless,
//     system::system_game_states::{
//         state_camera::CameraState,
//         state_gizmos::GizmosState,
//         state_gui::{GUIState, GuiElement, GuiWindow},
//         state_input,
//     },
// };
// use ecs_system::global_ecs_system;
// use hecs::World;

// use crate::component::component_transform::Transform;

// #[global_ecs_system]
// pub struct SystemDebugGuiEntity {}
// impl SystemDebugGuiEntity {}
// impl SystemDebugGuiEntity {
//     pub fn new() -> Box<SystemDebugGuiEntity> {
//         Box::new(SystemDebugGuiEntity {})
//     }
// }
// impl ECSSystemEventless for SystemDebugGuiEntity {
//     fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
//         true
//     }

//     fn debug(&mut self, game_state: &mut GameState, world: &mut World, _: &mut EventQueue) {
//         // let state_camera = game_state.get_value2::<CameraState>();
//         // let state_input = game_state.get_value2::<state_input::InputState>();

//         // // let mut closest: Option<Entity> = None;
//         // let mut distance = 99999999.0;
//         // let mut screen_pos = Vector3::zero();
//         // let mut matrix = Matrix4x4::default();
//         // for (_, transform) in world.query::<&Transform>().iter() {
//         //     let Some(p) = state_camera.world_to_screen(transform.position) else {
//         //         continue;
//         //     };

//         //     let d = (Vector3::new(p.0, p.1, 0.0) - state_input.cursor.position).magnitude();
//         //     let is_less = d < distance;
//         //     if !is_less {
//         //         continue;
//         //     }

//         //     // closest = Some(entity);
//         //     distance = d;
//         //     screen_pos = Vector3::new(p.0, p.1, 0.0);
//         //     matrix = transform.get_matrix();
//         // }

//         // game_state.edit::<GUIState>(|x| {
//         //     let mut w = GuiWindow::new(String::from("cur"), screen_pos.clone(), Vector3::zero());
//         //     w.add(GuiElement::new_label(String::from("HERE"), 20.0, color::Color::red()));

//         //     x.guis.push(w);
//         // });

//         // game_state.edit::<GizmosState>(|x| {
//         //     x.draw_calls
//         //         .push(Gizmo::cube(matrix, Vector3::one() * 3.0, color::Color::blue()));
//         // });
//     }
// }
