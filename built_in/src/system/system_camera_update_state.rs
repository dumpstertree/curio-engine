use core::{
    collections::{event_queue::EventQueue2, game_state::GameState},
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
    system::system_game_states::state_debug::StateDebug,
};
use ecs_system::global_ecs_system;
use hecs::World;

use core::system::system_game_states::state_camera::CameraState;

use crate::component::{component_camera::Camera, component_transform::Transform};

#[global_ecs_system]
pub struct PostCameraECSSystem {}
impl PostCameraECSSystem {}
impl ECSSystemEventless for PostCameraECSSystem {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn tick(&mut self, state: &mut GameState, world: &mut World, _: &mut EventQueue2) {
        if state.get_value2::<StateDebug>().is_paused {
            return;
        }
        for (_, (t, _)) in world.query_mut::<(&mut Transform, &Camera)>() {
            state.edit::<CameraState>(|x| {
                x.position = t.position;
                x.rotation = t.rotation;
            });
        }
    }
}
