use built_in_state::{state_camera::CameraState, state_debug::StateDebug};
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};
use ecs_system::global_ecs_system;
use hecs::World;

use crate::component::{component_camera::Camera, component_transform::Transform};

#[global_ecs_system]
pub struct PostCameraECSSystem {}
impl PostCameraECSSystem {}
impl ECSSystemEventless for PostCameraECSSystem {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn tick(&mut self, state: &mut GameState, world: &mut World, _: &mut EventQueue) {
        if state.get::<StateDebug>().is_paused {
            return;
        }
        for (_, (transform, _camera)) in world.query_mut::<(&mut Transform, &Camera)>() {
            state.edit::<CameraState>(|x| {
                x.cameras.position = transform.position;
                x.cameras.rotation = transform.rotation;
            });
        }
    }
}
