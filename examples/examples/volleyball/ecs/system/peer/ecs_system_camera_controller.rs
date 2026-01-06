use built_in_state::state_time::TimeState;
use ecs_system::global_ecs_system;
use system_component_default_gameplay::{component::{component_camera::Camera, component_transform::Transform}, ecs_system::ECSSystemEventless, world_context::{WorldContext, WorldContextCommon}};

use core::{
    collections::{event_queue::EventQueue, game_state::GameState, quaternion::Quaternion, vector3::Vector3},
    dumpster_engine::NetworkModes,
};

use crate::{
    game_board::GameBoard,
    state::{
        peer::state_peer_select_targets::StatePeerSelectTargets,
        state_position_ball::StatePositionBall,
        state_position_player::StatePositionEntities,
        state_teams::Teams,
        state_turn::StateTurn,
    },
};

#[global_ecs_system]
pub struct ECSSystemPeerStart {}
impl ECSSystemEventless for ECSSystemPeerStart {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut WorldContext) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalPeer, NetworkModes::OnlinePeer]
    }
    fn init(&mut self, game_state: &mut GameState, world: &mut WorldContext, _: &mut EventQueue) {}
    fn enable(&mut self, game_state: &mut GameState, world: &mut WorldContext, event_queue: &mut EventQueue) {}
    fn did_tick(&mut self, game_state: &mut GameState, context: &mut WorldContext, _: &mut EventQueue) {
        // get state
        let state_select_target = game_state.get::<StatePeerSelectTargets>();
        let state_time = game_state.get::<TimeState>();
        let state_pos_ball = game_state.get::<StatePositionBall>();
        let state_pos_entity = game_state.get::<StatePositionEntities>();
        let state_turn = game_state.get::<StateTurn>();
        let Some(tile_player) = state_pos_entity.positions.get(&game_state.instance_id) else {
            return;
        };
        let pos_ball = GameBoard::get_world_position(state_pos_ball.column, state_pos_ball.row);
        let pos_player = GameBoard::get_world_position(tile_player.0, tile_player.1);

        let mut t = Vector3::zero();
        let mut i = 0.0;
        // for x in state_pos_entity.positions {
        //     let p = GameBoard::get_world_position(x.1.0, x.1.1);
        //     t = t + p;
        //     i += 1.0;
        // }
        for _ in 0..1 {
            t = t + pos_ball;
            i += 1.0;
        }
        for _ in 0..1 {
            t = t + pos_player;
            i += 1.0;
        }

        // let focus_pos = (pos_player + pos_ball) / 2.0;
        let focus_pos = t / i;

        context.query_mut::<(&Camera, &mut Transform)>(|x| {
            for (_, (_, t)) in x {
                let mut tar: Vector3;
                let rot: Quaternion;
                if state_turn.active_instance_id == Teams::Red {
                    if state_select_target.enabled.is_none() {
                        let d = (pos_ball - pos_player).z.clamp(0.0, 14.0);
                        tar = focus_pos + Vector3::new(0.0, 6.0, -(5.0 + d / 2.0));
                        if d == 0.0 {
                            tar.z = -13.0;
                        } else {
                            tar.z = -13.0 - (14.0 / d) * 1.0;
                        }
                        // tar.x = tar.x / 2.0;

                        // rot = Quaternion::from_euler(Vector3::new(30.0, 0.0, 0.0));
                        let mut dir_to = pos_ball - tar;
                        dir_to.y = 0.0;
                        dir_to.normalize();

                        // rot = Quaternion::from_euler(Vector3::new(30.0, dir_to.y, 0.0));

                        rot = Quaternion::from_look_rotation(dir_to, Vector3::up());

                        let mut rot3 = rot.to_euler();
                        rot3.clamp_y(-30.0, 30.0);

                        let rot = Quaternion::from_euler(rot3);

                        let rot2 = rot * Quaternion::from_euler(Vector3::new(30.0, 0.0, 0.0));

                        t.rotation = t.rotation.slerp(rot2, 0.05);
                        t.position = Vector3::lerp(t.position, tar, 0.05);
                    } else {
                        let pos = GameBoard::get_world_position(state_select_target.selected_index.x, state_select_target.selected_index.y);
                        tar = Vector3::new(0.0, 15.0, 0.0);
                        rot = Quaternion::from_euler(Vector3::new(90.0, 0.0, 0.0));

                        t.rotation = t.rotation.slerp(rot, 0.3);
                        t.position = Vector3::lerp(t.position, tar, 0.3);
                    }
                } else {
                    let rot2 = Quaternion::from_euler(Vector3::new(30.0, 90.0, 0.0));

                    t.rotation = t.rotation.slerp(rot2, 0.5);
                    t.position = Vector3::lerp(t.position, Vector3::new(-10.0, 6.0, 0.0), 0.5);
                }
                // t.position.x = 0.0 + (sin * 0.25) as f32;
                // t.position.y = 6.0 + (sin * 0.25) as f32;
                // t.position.z = -14.0 + (sin * 0.25) as f32;
            }
            //
        });
    }
}
