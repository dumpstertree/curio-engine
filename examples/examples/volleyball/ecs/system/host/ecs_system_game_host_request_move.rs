use crate::{
    game_events::GameEvents,
    state::{state_energy::StateEnergy, state_position_player::StatePositionPlayer, state_turn::StateTurn},
};
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::{ecs_event_reciever, ecs_system::ECSSystemEventless},
};
use ecs_event::global_ecs_system_event_reciever;
use ecs_system::global_ecs_system;
use hecs::World;

#[global_ecs_system]
pub struct ECSSystemGameRequestMove {}
impl ECSSystemGameRequestMove {
    fn check_energy(game_state: &mut GameState, id: i32) -> bool {
        let has_energy = game_state.get_value2::<StateEnergy>().all_players[&id].0 > 0;
        if !has_energy {
            println!("Requested Move for not enough energy");
            return false;
        }

        return true;
    }
    fn check_player_id(game_state: &mut GameState, id: i32) -> bool {
        let is_active_player = game_state.get_value2::<StateTurn>().active_instance_id == id;
        if !is_active_player {
            println!("Requested Move for non-active player");
            return false;
        }

        return true;
    }
    fn check_bounds(game_state: &mut GameState, id: i32, x_diff: i32, z_diff: i32) -> bool {
        let cur_pos = game_state.get_value2::<StatePositionPlayer>().positions[&id];
        let new_pos = (cur_pos.0 + x_diff, cur_pos.1 + z_diff);
        let in_bounds = new_pos.0 >= 0 && new_pos.0 <= 3 && new_pos.1 >= 0 && new_pos.1 <= 1;
        if !in_bounds {
            println!("Requested Move out of bounds. ({},{}) -> ({},{})", cur_pos.0, cur_pos.1, new_pos.0, new_pos.1);
            return false;
        }
        println!(" Move bounds. ({},{}) -> ({},{})", cur_pos.0, cur_pos.1, new_pos.0, new_pos.1);

        return true;
    }
}
impl ECSSystemEventless for ECSSystemGameRequestMove {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
#[global_ecs_system_event_reciever(GameEvents)]
impl ecs_event_reciever::EventReciever<GameEvents> for ECSSystemGameRequestMove {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut World, _: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::RequestMoveZPos(id) => {
                if !ECSSystemGameRequestMove::check_player_id(game_state, *id) {
                    return;
                }
                if !ECSSystemGameRequestMove::check_energy(game_state, *id) {
                    return;
                }
                if !ECSSystemGameRequestMove::check_bounds(game_state, *id, 0, 1) {
                    return;
                }

                game_state.edit::<StatePositionPlayer>(|x| {
                    x.positions
                        .insert(*id, (x.positions[id].0, x.positions[id].1 + 1));
                });
                game_state.edit::<StateEnergy>(|x| {
                    x.all_players
                        .insert(*id, (x.all_players[id].0 - 1, x.all_players[id].1));
                });
            }
            GameEvents::RequestMoveZNeg(id) => {
                if !ECSSystemGameRequestMove::check_player_id(game_state, *id) {
                    return;
                }
                if !ECSSystemGameRequestMove::check_energy(game_state, *id) {
                    return;
                }
                if !ECSSystemGameRequestMove::check_bounds(game_state, *id, 0, -1) {
                    return;
                }

                game_state.edit::<StatePositionPlayer>(|x| {
                    x.positions
                        .insert(*id, (x.positions[id].0, x.positions[id].1 - 1));
                });
                game_state.edit::<StateEnergy>(|x| {
                    x.all_players
                        .insert(*id, (x.all_players[id].0 - 1, x.all_players[id].1));
                });
            }
            GameEvents::RequestMoveXPos(id) => {
                if !ECSSystemGameRequestMove::check_player_id(game_state, *id) {
                    return;
                }
                if !ECSSystemGameRequestMove::check_energy(game_state, *id) {
                    return;
                }
                if !ECSSystemGameRequestMove::check_bounds(game_state, *id, 1, 0) {
                    return;
                }

                game_state.edit::<StatePositionPlayer>(|x| {
                    x.positions
                        .insert(*id, (x.positions[id].0 + 1, x.positions[id].1));
                });
                game_state.edit::<StateEnergy>(|x| {
                    x.all_players
                        .insert(*id, (x.all_players[id].0 - 1, x.all_players[id].1));
                });
            }
            GameEvents::RequestMoveXNeg(id) => {
                if !ECSSystemGameRequestMove::check_player_id(game_state, *id) {
                    return;
                }
                if !ECSSystemGameRequestMove::check_energy(game_state, *id) {
                    return;
                }
                if !ECSSystemGameRequestMove::check_bounds(game_state, *id, -1, 0) {
                    return;
                }

                game_state.edit::<StatePositionPlayer>(|x| {
                    x.positions
                        .insert(*id, (x.positions[id].0 - 1, x.positions[id].1));
                });
                game_state.edit::<StateEnergy>(|x| {
                    x.all_players
                        .insert(*id, (x.all_players[id].0 - 1, x.all_players[id].1));
                });
            }
            _ => {}
        }
    }
}
