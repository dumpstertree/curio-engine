use gameplay::{
    context_3d::Context3D,
    traits::{habit::Habit, scope::Scope},
    traits_internal::world_context_common::ContextCommon,
};
use habit::habit;

use curio_core::{
    built_in::record::{sys_record_camera::SysRecordCamera, sys_record_sun::SysRecordSun},
    collections::network_modes::NetworkModes,
    collections::{color::Color, event_queue::EventQueue, game_state::GameState, vector3::Vector3},
    io::asset_loader::AssetLoader,
};

use crate::{
    Assets,
    state::state_teams::{StateTeamAssignments, Teams},
};

#[habit]
pub struct Instance {}
impl Scope for Instance {
    fn is_enabled(&mut self, _game_state: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _game_state: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all_peer()
    }
}
impl Habit for Instance {
    fn init(&mut self, game_state: &mut GameState, _world: &mut Context3D, _: &mut EventQueue) {
        println!("Instance: {}. Peer Init", game_state.instance_id);
    }
    fn enable(&mut self, game_state: &mut GameState, world: &mut Context3D, _event_queue: &mut EventQueue) {
        println!("Instance: {}. Peer Startup", game_state.instance_id);

        // load any remote assets now
        AssetLoader::preload_remote_assets(false);

        // set resolution
        game_state.edit::<SysRecordCamera>(|x| {
            x.resolution_width = 1920 / 1;
            x.resolution_height = 1080 / 1;
        });
        game_state.edit::<SysRecordSun>(|x| {
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
            println!("Spawned Fallback Camera");
            world.spawn_prefab_recursive(&AssetLoader::load_prefab(&Assets::PrefabCamera.into()));
            return;
        };

        match team {
            Teams::Red => {
                println!("Spawned Red Camera");
                world.spawn_prefab_recursive(&AssetLoader::load_prefab(&Assets::PrefabCamera.into()));
            }
            Teams::Blue => {
                println!("Spawned Blue Camera");
                world.spawn_prefab_recursive(&AssetLoader::load_prefab(&Assets::PrefabCamera.into()));
            }
        }
    }
}
