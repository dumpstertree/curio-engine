use crate::ecs::components::component_player::ComponentPlayer;
use crate::ecs::components::component_view_player::ComponentViewPlayer;
use crate::game_board::GameBoard;
use crate::state::state_position_player::StatePositionEntities;
use built_in_state::state_time::TimeState;
use core::collections::quaternion::Quaternion;
use core::collections::vector3::Vector3;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_system::global_ecs_system;
use system_component_default_gameplay::component::component_renderer_animated::RendererAnimated;
use system_component_default_gameplay::component::component_transform::Transform;
use system_component_default_gameplay::ecs_system::ECSSystemEventless;
use system_component_default_gameplay::world_context::{WorldContext, WorldContextCommon};

#[global_ecs_system]
pub struct ECSSystemViewMovePlayers {}
impl ECSSystemEventless for ECSSystemViewMovePlayers {
    fn run_on_instance(&mut self, _game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        NetworkModes::all_peer()
    }
    fn is_enabled(&mut self, _game_state: &mut GameState, _: &mut WorldContext) -> bool {
        true
    }
    fn tick(&mut self, game_state: &mut GameState, world: &mut WorldContext, events: &mut EventQueue) {
        let state_position_player = game_state.get::<StatePositionEntities>();
        let state_time = game_state.get::<TimeState>();

        world.query_mut::<(&mut Transform, &ComponentPlayer, &ComponentViewPlayer, &mut RendererAnimated)>(|query| {
            for (_, (transform, player, _, renderer)) in query {
                let Some(loc) = state_position_player.positions.get(&player.player_id) else {
                    continue;
                };

                // get pos
                let cur_pos = transform.position;
                let tar_pos = GameBoard::get_world_position(loc.0, loc.1);

                //move towards position and get back the delta
                let move_delta = transform.move_towards_position(tar_pos, 10.0 * state_time.scaled_delta_time);
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
        });
    }
}
