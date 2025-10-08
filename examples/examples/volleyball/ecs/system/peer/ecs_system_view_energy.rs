use crate::AssetMappingUIDs;
use crate::ecs::components::component_ball::ComponentBall;
use crate::ecs::components::component_energy_token::ComponentEnergyToken;
use crate::ecs::components::component_player::ComponentPlayer;
use crate::game_board::GameBoard;
use crate::state::state_energy::StateEnergy;
use crate::state::state_position_ball::StatePositionBall;
use crate::state::state_teams::StateTeamAssignments;
use built_in::component::component_renderer_animated::RendererAnimated;
use built_in::component::component_renderer_static::Renderer;
use built_in::component::component_transform::Transform;
use built_in_state::state_camera::CameraState;
use built_in_state::state_time::TimeState;
use core::collections::quaternion::Quaternion;
use core::collections::vector3::{self, Vector3};
use core::io::asset_database::AssetDatabaseListing;
use core::io::asset_loader::AssetLoader;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};
use ecs_system::global_ecs_system;
use hecs::World;

#[global_ecs_system]
pub struct ECSSystemViewMoveBall {
    did_init: i32,
}
impl ECSSystemEventless for ECSSystemViewMoveBall {
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalPeer, NetworkModes::OnlinePeer]
    }
    fn is_enabled(&mut self, game_state: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn enable(&mut self, game_state: &mut GameState, world: &mut World, _: &mut EventQueue) {}

    fn tick(&mut self, game_state: &mut GameState, world: &mut World, events: &mut EventQueue) {
        if self.did_init == 15 {
            let asset = AssetLoader::load_model_animated_from_database(AssetMappingUIDs::EnergyToken.uid());
            for team in game_state
                .get_value2::<StateTeamAssignments>()
                .team_assignments
            {
                for player_id in team.1 {
                    for i in 0..9 {
                        let mut r = RendererAnimated::default();
                        r.set_fps(60).set_asset(Some(asset.clone()));
                        // r.set_animation("add", true);
                        world.spawn((ComponentEnergyToken::default().set_index(i), ComponentPlayer::default().set_player_id(player_id), Transform::default(), r));
                    }
                }
            }
        }
        self.did_init += 1;
        let state_camera = game_state.get_value2::<CameraState>();
        let state_energy = game_state.get_value2::<StateEnergy>();
        let state_team = game_state.get_value2::<StateTeamAssignments>();
        for (_, (energy, transform, player, renderer)) in world
            .query::<(&ComponentEnergyToken, &mut Transform, &ComponentPlayer, &mut RendererAnimated)>()
            .iter()
        {
            let mut z = 1.0;
            let mut x = 0.0;

            let mut y = 0.0 + (energy.index as f32) * 0.05;

            match state_team.team_for(&player.player_id).unwrap() {
                crate::state::state_teams::Teams::Red => x = -0.9,
                crate::state::state_teams::Teams::Blue => x = 0.9,
            };

            let Some(player_energy) = state_energy.all_players.get(&player.player_id) else {
                continue;
            };

            transform.scale = Vector3::one() * 0.05;
            transform.rotation = game_state.get_value2::<CameraState>().cameras.rotation * Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0));

            transform.position = state_camera.cameras.position + (state_camera.cameras.rotation * Vector3::forward()) * z + state_camera.cameras.rotation * Vector3::down() * y + state_camera.cameras.rotation * Vector3::right() * x;
            if energy.index < player_energy.0 {
                renderer.set_animation("add", false);
            } else {
                // transform.position = Vector3::zero();
                renderer.set_animation("remove", false);
            }
        }
    }
}
