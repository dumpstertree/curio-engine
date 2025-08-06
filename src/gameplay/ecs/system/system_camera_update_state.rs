use crate::{
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
    system::system_game_states::state_debug::StateDebug,
    Collections::{event_queue::EventQueue2, game_state::GameState},
};
use ecs_event::ECSEvent;
use ecs_system::ECSSystem;
use hecs::World;

use crate::{
    gameplay::ecs::component::{component_camera::Camera, component_transform::Transform},
    system::system_game_states::state_camera::CameraState,
    IO::AssetLoader::AssetLoader,
};

#[ECSSystem]
pub struct PostCameraECSSystem {}
impl PostCameraECSSystem {}
impl ECSSystemEventless for PostCameraECSSystem {
    fn is_enabled(&mut self, game_state: &mut GameState, world: &mut World) -> bool {
        true
    }
    fn tick(&mut self, state: &mut GameState, world: &mut World, events: &mut EventQueue2) {
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
