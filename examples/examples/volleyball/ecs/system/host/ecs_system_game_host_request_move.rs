use crate::{
    game_board::GameBoard,
    game_events::GameEvents,
    state::{state_energy::StateEnergy, state_position_player::StatePositionEntities, state_teams::StateTeamAssignments, state_turn::StateTurn},
};
use curio_core::{
    collections::{event_queue::EventQueue, game_state::GameState, vector2_int::Vector2Int},
    dumpster_engine::NetworkModes,
};
use gameplay::{
    context_3d::Context3D,
    traits::{impulse::Impulse, scope::Scope},
};
use impulse::impulse;

#[derive(Default)]
#[impulse(GameEvents)]
pub struct ECSSystemGameRequestMove {}
impl ECSSystemGameRequestMove {
    fn check_energy(game_state: &mut GameState, id: i32) -> bool {
        let has_energy = game_state.get::<StateEnergy>().all_players[&id].0 > 0;
        if !has_energy {
            println!("Requested Move for not enough energy");
            return false;
        }

        return true;
    }
    fn check_player_id(game_state: &mut GameState, id: i32) -> bool {
        let state_teams = game_state
            .get::<StateTeamAssignments>()
            .team_for(&id)
            .unwrap();
        let is_active_player = game_state.get::<StateTurn>().active_instance_id == state_teams;
        if !is_active_player {
            println!("Requested Move for non-active player");
            return false;
        }

        return true;
    }
    fn check_bounds(game_state: &mut GameState, id: i32, x_diff: i32, z_diff: i32, bounds_min: Vector2Int, bounds_max: Vector2Int) -> bool {
        let cur_pos = game_state.get::<StatePositionEntities>().positions[&id];
        let new_pos = (cur_pos.0 + x_diff, cur_pos.1 + z_diff);
        let in_bounds = new_pos.0 >= bounds_min.x && new_pos.0 <= bounds_max.x && new_pos.1 >= bounds_min.y && new_pos.1 <= bounds_max.y;
        if !in_bounds {
            println!("Requested Move out of bounds. ({},{}) -> ({},{})", cur_pos.0, cur_pos.1, new_pos.0, new_pos.1);
            return false;
        }
        println!(" Move bounds. ({},{}) -> ({},{})", cur_pos.0, cur_pos.1, new_pos.0, new_pos.1);

        return true;
    }
}
impl Scope for ECSSystemGameRequestMove {
    fn is_enabled(&mut self, _: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
impl Impulse<GameEvents> for ECSSystemGameRequestMove {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut Context3D, _: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::RequestMoveZPos(id) => {
                if !ECSSystemGameRequestMove::check_player_id(game_state, *id) {
                    return;
                }
                if !ECSSystemGameRequestMove::check_energy(game_state, *id) {
                    return;
                }

                let Some(team) = game_state.get::<StateTeamAssignments>().team_for(&id) else {
                    return;
                };

                let dir = team.convert_dir(0, 1);
                if !ECSSystemGameRequestMove::check_bounds(game_state, *id, dir.0, dir.1, GameBoard::get_bounds_min_for_team(&team), GameBoard::get_bounds_max_for_team(&team)) {
                    return;
                }

                game_state.edit::<StatePositionEntities>(|x| {
                    x.positions
                        .insert(*id, (x.positions[id].0 + dir.0, x.positions[id].1 + dir.1));
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
                let Some(team) = game_state.get::<StateTeamAssignments>().team_for(&id) else {
                    return;
                };
                let dir = team.convert_dir(0, -1);
                if !ECSSystemGameRequestMove::check_bounds(game_state, *id, dir.0, dir.1, GameBoard::get_bounds_min_for_team(&team), GameBoard::get_bounds_max_for_team(&team)) {
                    return;
                }

                game_state.edit::<StatePositionEntities>(|x| {
                    x.positions
                        .insert(*id, (x.positions[id].0 + dir.0, x.positions[id].1 + dir.1));
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
                let Some(team) = game_state.get::<StateTeamAssignments>().team_for(&id) else {
                    return;
                };

                let dir = team.convert_dir(1, 0);
                if !ECSSystemGameRequestMove::check_bounds(game_state, *id, dir.0, dir.1, GameBoard::get_bounds_min_for_team(&team), GameBoard::get_bounds_max_for_team(&team)) {
                    return;
                }

                game_state.edit::<StatePositionEntities>(|x| {
                    x.positions
                        .insert(*id, (x.positions[id].0 + dir.0, x.positions[id].1 + dir.1));
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
                let Some(team) = game_state.get::<StateTeamAssignments>().team_for(&id) else {
                    return;
                };

                let dir = team.convert_dir(-1, 0);
                if !ECSSystemGameRequestMove::check_bounds(game_state, *id, dir.0, dir.1, GameBoard::get_bounds_min_for_team(&team), GameBoard::get_bounds_max_for_team(&team)) {
                    return;
                }

                game_state.edit::<StatePositionEntities>(|x| {
                    x.positions
                        .insert(*id, (x.positions[id].0 + dir.0, x.positions[id].1 + dir.1));
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
