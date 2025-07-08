use hecs::World;

use crate::{
    gameplay::{
        ecs::component::{component_renderer::Renderer, component_transform::Transform},
        game_events::GameEvents,
    },
    my_game::ecs::component::component_spin::Spin,
    system::system_components::gameplay_components::gameplay_component_default::{ECSSystem, EventQueue},
    Collections::{game_state::GameState, vector3::Vector3},
    IO::AssetLoader::AssetLoader,
};

pub struct SystemGameInit {}
impl SystemGameInit {
    pub fn new() -> Box<SystemGameInit> {
        Box::new(SystemGameInit {})
    }
}
impl ECSSystem<GameEvents> for SystemGameInit {
    fn is_enabled(&mut self, game_state: &mut GameState, world: &mut World, event_queue: &mut EventQueue<GameEvents>) -> bool {
        true
    }
    fn init(&mut self, game_state: &mut GameState, scene: &mut World, event_queue: &mut EventQueue<GameEvents>, asset_loader: &mut AssetLoader) {
        scene.spawn((
            Renderer::default().set_asset(asset_loader.load_gltf("Cube3.glb")),
            Transform::default()
                .set_position(Vector3::one())
                .set_scale(Vector3::one() * 2.0),
            Spin::default().set_axis(Vector3::up()).set_speed(-2.0),
        ));
        scene.spawn((
            Renderer::default().set_asset(asset_loader.load_gltf("Cube4.glb")),
            Transform::default().set_position(Vector3::one() * 3.0),
            Spin::default()
                .set_axis((Vector3::right() + Vector3::forward()).normalized())
                .set_speed(5.0),
        ));
    }
}
