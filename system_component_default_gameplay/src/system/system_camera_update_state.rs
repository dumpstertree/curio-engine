use built_in_state::{state_camera::CameraState, state_debug::StateDebug};
use core::{
    collections::{
        event_queue::EventQueue,
        game_state::{self, GameState},
    },
    gameplay::{
        // ecs::{component::component_transform::Transform, traits::ecs_system::ECSSystemEventless},
        // world_context::{WorldContext, WorldContextCommon},
    },
};
// use ecs_system::global_ecs_system;
use hecs::World;

use crate::{component::{component_camera::Camera, component_transform::Transform}, ecs_system::ECSSystemEventless, world_context::{WorldContext, WorldContextCommon}};

// #[global_ecs_system]
#[derive(Default)]

pub struct PostCameraECSSystem {}
impl PostCameraECSSystem {}
impl ECSSystemEventless for PostCameraECSSystem {
    fn is_enabled(&mut self, game_state: &mut GameState, _: &mut WorldContext) -> bool {
        true
    }
    fn tick(&mut self, state: &mut GameState, world: &mut WorldContext, _: &mut EventQueue) {
        if state.get::<StateDebug>().is_paused {
            return;
        }
        world.query_mut::<(&mut Transform, &Camera)>(|q| {
            //
            for (_entity, (transform, _camera)) in q {
                state.edit::<CameraState>(|x| {
                    x.cameras.position = transform.position;
                    x.cameras.rotation = transform.rotation;
                });
            }
        });
        // for (_, (transform, _camera)) in world.query_mut::<(&mut Transform, &Camera)>() {
        //     state.edit::<CameraState>(|x| {
        //         x.cameras.position = transform.position;
        //         x.cameras.rotation = transform.rotation;
        //     });
        // }
    }
}
