use built_in::component::{
    component_camera::Camera, component_colliders::component_collider_box::ComponentColliderBox, component_renderer::Renderer,
    component_transform::Transform,
};
use ecs_system::global_ecs_system;
use hecs::World;

use crate::ecs::component::{component_ball::ComponentBall, component_paddle::ComponentPaddle};
use core::{
    collections::{event_queue::EventQueue2, game_state::GameState, vector3::Vector3},
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
    io::asset_loader::AssetLoader,
    random::Random,
    system::system_game_states::state_camera::CameraState,
};
#[global_ecs_system]
pub struct SystemPongInit {}
impl ECSSystemEventless for SystemPongInit {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn init(&mut self, state: &mut GameState, world: &mut World, _: &mut EventQueue2, asset_loader: &mut AssetLoader) {
        state.edit::<CameraState>(|x| {
            x.width = 1920;
            x.height = 1080;
        });

        // camera
        world.spawn((
            Transform::default().set_position(Vector3::new(0.0, 5.0, -20.0)),
            // .set_rotation(Quaternion::from_angle_axis(Vector3::new(0.0, 1.0, 0.0), 180.0)),
            Camera::default(),
        ));

        // paddle
        world.spawn((
            ComponentColliderBox::default().set_size(Vector3::new(3.0, 1.0, 1.0)),
            Renderer::default().set_asset(asset_loader.load_gltf("Cube3.glb")),
            Transform::default().set_position(Vector3::new(0.0, 0.0, -10.0)),
            ComponentPaddle::default()
                .set_axis(Vector3::right())
                .set_speed(5.0),
        ));
        world.spawn((
            ComponentColliderBox::default().set_size(Vector3::new(3.0, 1.0, 1.0)),
            Renderer::default().set_asset(asset_loader.load_gltf("Cube3.glb")),
            Transform::default().set_position(Vector3::new(0.0, 0.0, 10.0)),
            ComponentPaddle::default()
                .set_axis(Vector3::right())
                .set_speed(5.0),
        ));

        // ball
        world.spawn((
            ComponentColliderBox::default().set_size(Vector3::new(1.0, 1.0, 1.0)),
            Renderer::default().set_asset(asset_loader.load_gltf("Cone.glb")),
            Transform::default().set_position(Vector3::new(0.0, 0.0, 0.0)),
            ComponentBall::default()
                .set_axis(Random::direction(true, false, true))
                .set_speed(5.0),
        ));
    }
}
