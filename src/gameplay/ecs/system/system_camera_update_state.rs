use crate::Collections::game_state::GameState;
use hecs::World;

use crate::{
    gameplay::{
        ecs::component::{component_camera::Camera, component_transform::Transform},
        game_events::GameEvents,
    },
    system::{
        system_components::gameplay_components::gameplay_component_default::{ECSSystem, ECSSystemEventless, EventQueue},
        system_game_states::state_camera::CameraState,
    },
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
        for (_, (t, _)) in world.query_mut::<(&mut Transform, &Camera)>() {
            // set camera gamestate
            let mut state_camera = game_state.get_value2::<CameraState>();
            // state_camera.position = t.position.clone();
            state_camera.position = t.position;
            state_camera.rotation = t.rotation;

            // println!("pos: {}", state_camera.position);

            game_state.set_value2::<CameraState>(state_camera);
        }
    }
}
