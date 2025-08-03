use crate::{system::system_game_states::state_debug::StateDebug, Collections::game_state::GameState};
use hecs::World;

use crate::{
    gameplay::ecs::component::{component_camera::Camera, component_transform::Transform},
    system::{system_components::gameplay_components::gameplay_component_default::ECSSystemEventless, system_game_states::state_camera::CameraState},
    IO::AssetLoader::AssetLoader,
};

pub struct PostCameraECSSystem {}
impl PostCameraECSSystem {}
impl ECSSystemEventless for PostCameraECSSystem {
    fn is_enabled(&mut self, game_state: &mut GameState, world: &mut World) -> bool {
        true
    }
    fn init(&mut self, game_state: &mut GameState, scene: &mut World, asset_loader: &mut AssetLoader) {}
    fn tick(&mut self, game_state: &mut GameState, world: &mut World) {
        if game_state.get_value2::<StateDebug>().is_paused {
            return;
        }
        for (_, (t, _)) in world.query_mut::<(&mut Transform, &Camera)>() {
            game_state.edit::<CameraState>(|x| {
                x.position = t.position;
                x.rotation = t.rotation;
            });
        }
    }
}
