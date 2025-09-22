use built_in::component::{component_camera::Camera, component_renderer_animated::RendererAnimated, component_renderer_static::Renderer, component_transform::Transform};
use built_in_state::state_camera::CameraState;
use ecs_system::global_ecs_system;
use hecs::World;

use core::{
    collections::{event_queue::EventQueue, game_state::GameState, quaternion::Quaternion, vector3::Vector3},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
    io::asset_loader::AssetLoader,
};

use crate::state::state_teams::{StateTeamAssignments, Teams};

#[global_ecs_system]
pub struct ECSSystemPeerStart {}
impl ECSSystemEventless for ECSSystemPeerStart {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalPeer, NetworkModes::OnlinePeer]
    }
    fn enable(&mut self, game_state: &mut GameState, world: &mut World, _: &mut EventQueue) {
        println!("Instance: {}. Peer Startup", game_state.instance_id);

        // set resolution
        game_state.edit::<CameraState>(|x| {
            x.resolution_width = 1920 / 1;
            x.resolution_height = 1080 / 1;
        });

        let spine = AssetLoader::load_spine("path");
        world.spawn((
            Transform::default()
                .set_position(Vector3::new(0.0, -5.0, 0.0))
                .set_rotation(Quaternion::from_euler(Vector3::new(0.0, 0.0, 0.0))),
            RendererAnimated::default()
                .set_asset(Some(spine))
                .set_animation("walk", true)
                .set_skin("goblin"),
        ));
    }
    fn tick(&mut self, game_state: &mut GameState, world: &mut World, _: &mut EventQueue) {
        let Some(team) = game_state
            .get_value2::<StateTeamAssignments>()
            .team_for(&game_state.instance_id)
        else {
            println!("no team");
            return;
        };
        match team {
            Teams::Red => {
                world.spawn((
                    Transform::default()
                        .set_position(Vector3::new(0.0, 5.0, -9.0))
                        .set_rotation(Quaternion::from_euler(Vector3::new(30.0, 0.0, 0.0))),
                    Camera::default(),
                ));
            }
            Teams::Blue => {
                world.spawn((
                    Transform::default()
                        .set_position(Vector3::new(0.0, 5.0, 9.0))
                        .set_rotation(Quaternion::from_euler(Vector3::new(30.0, 180.0, 0.0))),
                    Camera::default(),
                ));
            }
        }
    }
}
