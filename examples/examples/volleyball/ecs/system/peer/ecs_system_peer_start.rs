use built_in_state::{state_camera::CameraState, state_sun::StateSun};
use ecs_system::habit;
use system_component_default_gameplay::{
    built_in::facet::{component_camera::Camera, facet_transform::component_transform::Transform},
    traits::{habit::Habit, scope::Scope},
    traits_internal::world_context_common::WorldContextCommon,
    world_context_3d::WorldContext,
};

use core::{
    collections::{color::Color, event_queue::EventQueue, game_state::GameState, quaternion::Quaternion, vector3::Vector3},
    dumpster_engine::NetworkModes,
    io::asset_loader::AssetLoader,
};

use crate::state::state_teams::{StateTeamAssignments, Teams};

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
    fn init(&mut self, game_state: &mut GameState, world: &mut WorldContext, _: &mut EventQueue) {
        println!("Instance: {}. Peer Init", game_state.instance_id);
    }
    fn enable(&mut self, game_state: &mut GameState, world: &mut WorldContext, event_queue: &mut EventQueue) {
        println!("Instance: {}. Peer Startup", game_state.instance_id);

        // load any remote assets now
        AssetLoader::preload_remote_assets(false);

        let p = world.instantiate_prefab(&AssetLoader::load_prefab());

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

            let a = world.instantiate(
                "camera",
                Transform::default()
                    .set_position(Vector3::new(0.0, 6.0, -14.0))
                    .set_rotation(Quaternion::from_euler(Vector3::new(30.0, 0.0, 0.0))),
            );

            a.add_component_value(
                // add camera
                Camera::default(),
            );

            return;
        };

        match team {
            Teams::Red => {
                let a = world.instantiate(
                    "camera",
                    Transform::default()
                        .set_position(Vector3::new(0.0, 6.0, -14.0))
                        .set_rotation(Quaternion::from_euler(Vector3::new(30.0, 0.0, 0.0))),
                );
                a.add_component_value(
                    // add camera
                    Camera::default(),
                );
            }
            Teams::Blue => {
                let a = world.instantiate(
                    "camera",
                    Transform::default()
                        .set_position(Vector3::new(0.0, 6.0, 14.0))
                        .set_rotation(Quaternion::from_euler(Vector3::new(30.0, 180.0, 0.0))),
                );
                a.add_component_value(
                    // add camera
                    Camera::default(),
                );
            }
        }
    }
}
