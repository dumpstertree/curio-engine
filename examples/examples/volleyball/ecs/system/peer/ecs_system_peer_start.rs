use gameplay::{
    context_3d::Context3D,
    traits::{habit::Habit, scope::Scope},
    traits_internal::world_context_common::ContextCommon,
};
use habit::habit;

use curio_core::{
    Color, PrefabGameObject, Severity, Vector3,
    built_in::record::{sys_record_camera::SysRecordCamera, sys_record_sun::SysRecordSun},
    collections::{event_queue::Nerve, ledger::Ledger},
    io::asset_loader::AssetLoader,
    network_modes::NetworkModes,
};

use crate::{
    Assets,
    state::state_teams::{StateTeamAssignments, Teams},
};

#[habit]
pub struct Instance {}
impl Scope for Instance {
    fn is_enabled(&mut self, _ledger: &mut Ledger) -> bool {
        true
    }
    fn run_on_instance(&mut self, _ledger: &mut Ledger) -> Vec<NetworkModes> {
        NetworkModes::all_peer()
    }
}
impl Habit for Instance {
    fn init(&mut self, ledger: &mut Ledger, _world: &mut Context3D, _: &mut Nerve) {
        ledger.log(Severity::Info, "Init");
    }
    fn enable(&mut self, ledger: &mut Ledger, world: &mut Context3D, _event_queue: &mut Nerve) {
        ledger.log(Severity::Info, "Enabled");

        // load any remote assets now
        AssetLoader::preload_remote_assets(false);

        // set resolution
        ledger.write::<SysRecordCamera>(|x| {
            x.resolution_width = 1920 / 1;
            x.resolution_height = 1080 / 1;
        });
        ledger.write::<SysRecordSun>(|x| {
            x.cast_shadows = true;
            x.color = Color::white();
            x.direction = (Vector3::forward() + Vector3::down()).normalize_and_copy();
        });

        // }
        // fn tick(&mut self, ledger: &mut GameState, world: &mut WorldContext, _: &mut EventQueue) {
        let Some(team) = ledger
            .read::<StateTeamAssignments>()
            .team_for(&ledger.network.me().guid)
        else {
            ledger.log(Severity::Info, "Spawned Fallback Camera");
            world.spawn_prefab_recursive(&AssetLoader::load_asset::<PrefabGameObject>(&Assets::PrefabCamera.into()));
            return;
        };

        match team {
            Teams::Red => {
                ledger.log(Severity::Info, "Spawned Red Camera");
                world.spawn_prefab_recursive(&AssetLoader::load_asset::<PrefabGameObject>(&Assets::PrefabCamera.into()));
            }
            Teams::Blue => {
                ledger.log(Severity::Info, "Spawned Blue Camera");
                world.spawn_prefab_recursive(&AssetLoader::load_asset::<PrefabGameObject>(&Assets::PrefabCamera.into()));
            }
        }
    }
}
