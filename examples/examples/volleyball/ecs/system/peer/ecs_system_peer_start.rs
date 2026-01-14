use built_in_state::{state_camera::CameraState, state_sun::StateSun};
use ecs_system::habit;
use system_component_default_gameplay::{
    built_in::facet::{camera::Camera, transform::transform3d::Transform3D},
    context_3d::Context3D,
    traits::{habit::Habit, scope::Scope},
    traits_internal::world_context_common::ContextCommon,
};

use curio_core::{
    collections::{color::Color, event_queue::EventQueue, game_state::GameState, quaternion::Quaternion, vector3::Vector3},
    dumpster_engine::NetworkModes,
    io::{asset::Asset, asset_database::AssetDatabaseListing, asset_loader::AssetLoader},
};

use crate::{
    Assets,
    state::state_teams::{StateTeamAssignments, Teams},
};

#[habit]
pub struct Instance {}
impl Scope for Instance {
    fn is_enabled(&mut self, game_state: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all_peer()
    }
}
impl Habit for Instance {
    fn init(&mut self, game_state: &mut GameState, world: &mut Context3D, _: &mut EventQueue) {
        println!("Instance: {}. Peer Init", game_state.instance_id);
    }
    fn enable(&mut self, game_state: &mut GameState, world: &mut Context3D, event_queue: &mut EventQueue) {
        println!("Instance: {}. Peer Startup", game_state.instance_id);

        // load any remote assets now
        AssetLoader::preload_remote_assets(false);

        // let p = world.spawn_prefab_recursive(&AssetLoader::load_prefab(&Assets::PrefabCamera.into()));

        // set resolution
        game_state.edit::<CameraState>(|x| {
            x.resolution_width = 1920 / 1;
            x.resolution_height = 1080 / 1;
        });
        game_state.edit::<StateSun>(|x| {
            x.cast_shadows = true;
            x.color = Color::white();
            x.direction = (Vector3::forward() + Vector3::down()).normalize_and_copy();
        });

        // }
        // fn tick(&mut self, game_state: &mut GameState, world: &mut WorldContext, _: &mut EventQueue) {
        let Some(team) = game_state
            .get::<StateTeamAssignments>()
            .team_for(&game_state.instance_id)
        else {
            // fallback camera

            world
                .spawn(
                    "camera",
                    Transform3D::default()
                        .set_position(Vector3::new(0.0, 6.0, -14.0))
                        .set_rotation(Quaternion::from_euler(Vector3::new(30.0, 0.0, 0.0))),
                )
                .add_facet(
                    // add camera
                    Camera::default(),
                );

            println!("Spawned Fallback Camera");

            return;
        };

        match team {
            Teams::Red => {
                world
                    .spawn(
                        "camera",
                        Transform3D::default()
                            .set_position(Vector3::new(0.0, 6.0, -14.0))
                            .set_rotation(Quaternion::from_euler(Vector3::new(30.0, 0.0, 0.0))),
                    )
                    .add_facet(
                        // add camera
                        Camera::default(),
                    );

                println!("Spawned Red Camera");
            }
            Teams::Blue => {
                world
                    .spawn(
                        "camera",
                        Transform3D::default()
                            .set_position(Vector3::new(0.0, 6.0, 14.0))
                            .set_rotation(Quaternion::from_euler(Vector3::new(30.0, 180.0, 0.0))),
                    )
                    .add_facet(
                        // add camera
                        Camera::default(),
                    );
                println!("Spawned Blue Camera");
            }
        }
    }
}
