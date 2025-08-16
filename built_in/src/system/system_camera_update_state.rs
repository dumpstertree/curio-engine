use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
    system::system_game_states::state_debug::StateDebug,
};
use ecs_system::global_ecs_system;
use hecs::World;

use core::system::system_game_states::state_camera::CameraState;

use crate::component::{component_camera::Camera, component_camera_index::CameraIndex, component_transform::Transform};

#[global_ecs_system]
pub struct PostCameraECSSystem {}
impl PostCameraECSSystem {}
impl ECSSystemEventless for PostCameraECSSystem {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn tick(&mut self, state: &mut GameState, world: &mut World, _: &mut EventQueue) {
        if state.get_value2::<StateDebug>().is_paused {
            return;
        }
        for (_, (transform, _camera, camera_index)) in world.query_mut::<(&mut Transform, &Camera, &CameraIndex)>() {
            state.edit::<CameraState>(|x| {
                let Some(cam) = x.cameras.get_mut(camera_index.index) else {
                    println!(
                        "Attempting to write to Camera at index {} but only has length of {}",
                        camera_index.index,
                        x.cameras.len()
                    );
                    return;
                };

                cam.position = transform.position;
                cam.rotation = transform.rotation;
            });
        }
    }
}
