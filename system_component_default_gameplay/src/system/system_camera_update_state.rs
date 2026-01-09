use built_in_state::{state_camera::CameraState, state_debug::StateDebug};
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
// use ecs_system::habit;

use crate::{
    component::{component_camera::Camera, component_transform::Transform},
    traits::{habit::Habit, scope::Scope},
    world_context_common::WorldContextCommon,
    world_context_3d::WorldContext,
};

// #[global_ecs_system]
#[derive(Default)]
pub struct Instance {}
impl Scope for Instance {
    fn is_enabled(&mut self, _game_state: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _game_state: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all()
    }
}
impl Habit for Instance {
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
