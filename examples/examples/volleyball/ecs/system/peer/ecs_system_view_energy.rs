use crate::AssetMappingUIDs;
use crate::ecs::components::component_energy_token::ComponentEnergyToken;
use crate::ecs::components::component_player::ComponentPlayer;
use crate::state::state_energy::StateEnergy;
use crate::state::state_teams::StateTeamAssignments;
use built_in::component::component_renderer_animated::RendererAnimated;
use built_in::component::component_transform::Transform;
use built_in_state::state_camera::CameraState;
use core::collections::quaternion::Quaternion;
use core::collections::vector3::Vector3;
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
    fn tick(&mut self, game_state: &mut GameState, world: &mut World, events: &mut EventQueue) {
        self.did_init += 1;
        let state_camera = game_state.get::<CameraState>();
        let state_energy = game_state.get::<StateEnergy>();
        let state_team = game_state.get::<StateTeamAssignments>();
        for (_, (energy, transform, player, renderer)) in world
            .query::<(&ComponentEnergyToken, &mut Transform, &ComponentPlayer, &mut RendererAnimated)>()
            .iter()
        {
            let z = 1.0;
            let mut x = 0.0;

            let y = 0.0 + (energy.index as f32) * 0.05;

            match state_team.team_for(&player.player_id).unwrap() {
                crate::state::state_teams::Teams::Red => x = -0.9,
                crate::state::state_teams::Teams::Blue => x = 0.9,
            };

            let Some(player_energy) = state_energy.all_players.get(&player.player_id) else {
                continue;
            };

            transform.scale = Vector3::one() * 0.05;
            transform.rotation = game_state.get::<CameraState>().cameras.rotation * Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0));

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
