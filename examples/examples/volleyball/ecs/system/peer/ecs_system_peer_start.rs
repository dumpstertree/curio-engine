use built_in::component::{component_camera::Camera, component_light::ComponentLight, component_renderer_animated::RendererAnimated, component_renderer_static::Renderer, component_transform::Transform};
use built_in_state::{state_camera::CameraState, state_network::StateNetwork, state_sun::StateSun};
use ecs_system::global_ecs_system;
use hecs::World;

use core::{
    collections::{
        color::{self, Color},
        event_queue::EventQueue,
        game_state::GameState,
        quaternion::Quaternion,
        vector3::Vector3,
    },
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
    io::asset_loader::AssetLoader,
    random::Random,
};

use crate::{
    AssetMappingUIDs,
    ecs::components::{component_ball::ComponentBall, component_player::ComponentPlayer, component_view_player::ComponentViewPlayer},
    state::state_teams::{StateTeamAssignments, Teams},
};

#[global_ecs_system]
pub struct ECSSystemPeerStart {}
impl ECSSystemEventless for ECSSystemPeerStart {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalPeer, NetworkModes::OnlinePeer]
    }
    fn init(&mut self, _: &mut GameState, world: &mut World, _: &mut EventQueue) {}
    fn enable(&mut self, game_state: &mut GameState, world: &mut World, _: &mut EventQueue) {
        println!("Instance: {}. Peer Startup", game_state.instance_id);

        // set resolution
        game_state.edit::<CameraState>(|x| {
            x.resolution_width = 1920 / 1;
            x.resolution_height = 1080 / 1;
        });
        game_state.edit::<StateSun>(|x| {
            x.cast_shadows = true;
            x.color = Color::green();
            x.direction = (Vector3::forward() + Vector3::down()).normalize_and_copy();
        });

        // let mut l = ComponentLight::default();
        // l.color = Color::red();
        // world.spawn((Transform::default(), l));

        // let spine = AssetLoader::load_spine_from_path("path");
        let asset_goblin = AssetLoader::load_model_animated_from_database(AssetMappingUIDs::Goblin.uid());
        let asset_court = AssetLoader::load_model_static_from_database(AssetMappingUIDs::Court.uid());

        // court
        world.spawn((
            Transform::default()
                .set_position(Vector3::new(0.0, 0.0, 0.0))
                .set_rotation(Quaternion::from_euler(Vector3::new(0.0, 90.0, 0.0))),
            Renderer::default().set_asset(Some(asset_court)),
        ));
        for id in game_state.get_value2::<StateNetwork>().peer_instance_ids() {
            let mut rend = RendererAnimated::default();
            rend.set_asset(Some(asset_goblin.clone()))
                .set_animation("walk", true)
                .set_skin("goblin");
            // players
            world.spawn((
                ComponentViewPlayer::default(),
                ComponentPlayer::default().set_player_id(*id),
                Transform::default()
                    .set_position(Vector3::new(-5.0, -5.0, 10.0))
                    .set_rotation(Quaternion::from_euler(Vector3::new(1.0, 0.0, 1.0))),
                rend,
            ));
        }
        {
            let mut rend = RendererAnimated::default();
            rend.set_asset(Some(asset_goblin.clone()))
                .set_animation("walk", true)
                .set_skin("goblin");
            // players
            world.spawn((ComponentBall::default(), Transform::default(), rend));
        }

        // lighting
        world.spawn((
            Transform::default()
                .set_position(Vector3::new(0.0, 0.0, 0.0))
                .set_rotation(Quaternion::from_euler(Vector3::new(1.0, 0.0, 1.0))),
            ComponentLight::default(),
        ));

        game_state.edit::<StateSun>(|x| {
            x.cast_shadows = true;
            x.color = Color::white();
        });
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
                        .set_position(Vector3::new(0.0, 6.0, -14.0))
                        .set_rotation(Quaternion::from_euler(Vector3::new(30.0, 0.0, 0.0))),
                    Camera::default(),
                ));
            }
            Teams::Blue => {
                world.spawn((
                    Transform::default()
                        .set_position(Vector3::new(0.0, 6.0, 14.0))
                        .set_rotation(Quaternion::from_euler(Vector3::new(30.0, 180.0, 0.0))),
                    Camera::default(),
                ));
            }
        }

        let cam = game_state.get_value2::<CameraState>().cameras.rotation * Vector3::forward();
        game_state.edit::<StateSun>(|x| {
            x.direction = cam;
        });
    }
}
