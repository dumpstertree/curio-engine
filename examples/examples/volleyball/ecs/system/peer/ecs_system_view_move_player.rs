use crate::AssetMappingUIDs;
use crate::ecs::components::component_player::ComponentPlayer;
use crate::ecs::components::component_view_player::ComponentViewPlayer;
use crate::game_board::GameBoard;
use crate::state::state_position_player::StatePositionPlayer;
use built_in::component::component_renderer_animated::RendererAnimated;
use built_in::component::component_transform::Transform;
use built_in_state::state_network::StateNetwork;
use built_in_state::state_time::TimeState;
use core::collections::quaternion::Quaternion;
use core::collections::vector3::Vector3;
use core::io::asset_loader::AssetLoader;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};
use ecs_system::global_ecs_system;
use hecs::World;

#[global_ecs_system]
pub struct ECSSystemViewMovePlayers {}
impl ECSSystemEventless for ECSSystemViewMovePlayers {
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalPeer, NetworkModes::OnlinePeer]
    }
    fn is_enabled(&mut self, game_state: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn enable(&mut self, game_state: &mut GameState, world: &mut World, _: &mut EventQueue) {
        let asset_goblin = AssetLoader::load_model_animated_from_database(AssetMappingUIDs::Goblin.uid());

        for id in game_state.get_value2::<StateNetwork>().peer_instance_ids() {
            println!("spawn player");
            let mut rend = RendererAnimated::default();
            rend.set_asset(Some(asset_goblin.clone()));
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
    }
    fn tick(&mut self, game_state: &mut GameState, world: &mut World, events: &mut EventQueue) {
        let state_position_player = game_state.get_value2::<StatePositionPlayer>();
        let state_time = game_state.get_value2::<TimeState>();

        for (_, (transform, player, _, renderer)) in world
            .query::<(&mut Transform, &ComponentPlayer, &ComponentViewPlayer, &mut RendererAnimated)>()
            .iter()
        {
            let Some(loc) = state_position_player.positions.get(&player.player_id) else {
                continue;
            };

            // get pos
            let cur_pos = transform.position;
            let tar_pos = GameBoard::get_world_position(loc.0, loc.1);

            //move towards position and get back the delta
            let move_delta = transform.move_towards_position(tar_pos, 5.0 * state_time.scaled_delta_time);
            if move_delta > 0.0 {
                // rotate mesh based on direction
                let move_left = (tar_pos - cur_pos).x > 0.0;
                if move_left {
                    transform.rotation = Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0));
                } else {
                    transform.rotation = Quaternion::from_euler(Vector3::new(0.0, 0.0, 0.0));
                }

                // apply walking animation
                renderer.set_animation("run", true);
            } else {
                // clear animations
                renderer.set_animation("idle", true);
            }
        }
    }
}
