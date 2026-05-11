use crate::{
    game_board::GameBoard,
    game_events::GameEvents,
    state::{state_energy::StateEnergy, state_position_player::StatePositionEntities, state_teams::StateTeamAssignments, state_turn::StateTurn},
};
use curio_core::{
    Vector2Int,
    collections::{event_queue::Nerve, ledger::Ledger},
    network_modes::NetworkModes,
};
use gameplay::{
    context_3d::Context3D,
    traits::{impulse::Impulse, scope::Scope},
};
use impulse::impulse;

#[derive(Default)]
#[impulse(GameEvents)]
pub struct ECsystemGameRequestMove {}
impl ECsystemGameRequestMove {
    fn check_energy(ledger: &mut Ledger, id: i32) -> bool {
        let has_energy = ledger.read::<StateEnergy>().all_players[&id].0 > 0;
        if !has_energy {
            println!("Requested Move for not enough energy");
            return false;
        }

        return true;
    }
    fn check_player_id(ledger: &mut Ledger, id: i32) -> bool {
        let state_teams = ledger.read::<StateTeamAssignments>().team_for(&id).unwrap();
        let is_active_player = ledger.read::<StateTurn>().active_instance_id == state_teams;
        if !is_active_player {
            println!("Requested Move for non-active player");
            return false;
        }

        return true;
    }
    fn check_bounds(ledger: &mut Ledger, id: i32, x_diff: i32, z_diff: i32, bounds_min: Vector2Int, bounds_max: Vector2Int) -> bool {
        let cur_pos = ledger.read::<StatePositionEntities>().positions[&id];
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
impl Scope for ECsystemGameRequestMove {
    fn is_enabled(&mut self, _: &mut Ledger) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut Ledger) -> Vec<NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
impl Impulse<GameEvents> for ECsystemGameRequestMove {
    fn dequeue_event(&mut self, ledger: &mut Ledger, _: &mut Context3D, _: &mut Nerve, event: &GameEvents) {
        match event {
            GameEvents::RequestMoveZPos(id) => {
                if !ECsystemGameRequestMove::check_player_id(ledger, *id) {
                    return;
                }
                if !ECsystemGameRequestMove::check_energy(ledger, *id) {
                    return;
                }

                let Some(team) = ledger.read::<StateTeamAssignments>().team_for(&id) else {
                    return;
                };

                let dir = team.convert_dir(0, 1);
                if !ECsystemGameRequestMove::check_bounds(ledger, *id, dir.0, dir.1, GameBoard::get_bounds_min_for_team(&team), GameBoard::get_bounds_max_for_team(&team)) {
                    return;
                }

                ledger.write::<StatePositionEntities>(|x| {
                    x.positions
                        .insert(*id, (x.positions[id].0 + dir.0, x.positions[id].1 + dir.1));
                });
                ledger.write::<StateEnergy>(|x| {
                    x.all_players
                        .insert(*id, (x.all_players[id].0 - 1, x.all_players[id].1));
                });
            }
            GameEvents::RequestMoveZNeg(id) => {
                if !ECsystemGameRequestMove::check_player_id(ledger, *id) {
                    return;
                }
                if !ECsystemGameRequestMove::check_energy(ledger, *id) {
                    return;
                }
                let Some(team) = ledger.read::<StateTeamAssignments>().team_for(&id) else {
                    return;
                };
                let dir = team.convert_dir(0, -1);
                if !ECsystemGameRequestMove::check_bounds(ledger, *id, dir.0, dir.1, GameBoard::get_bounds_min_for_team(&team), GameBoard::get_bounds_max_for_team(&team)) {
                    return;
                }

                ledger.write::<StatePositionEntities>(|x| {
                    x.positions
                        .insert(*id, (x.positions[id].0 + dir.0, x.positions[id].1 + dir.1));
                });
                ledger.write::<StateEnergy>(|x| {
                    x.all_players
                        .insert(*id, (x.all_players[id].0 - 1, x.all_players[id].1));
                });
            }
            GameEvents::RequestMoveXPos(id) => {
                if !ECsystemGameRequestMove::check_player_id(ledger, *id) {
                    return;
                }
                if !ECsystemGameRequestMove::check_energy(ledger, *id) {
                    return;
                }
                let Some(team) = ledger.read::<StateTeamAssignments>().team_for(&id) else {
                    return;
                };

                let dir = team.convert_dir(1, 0);
                if !ECsystemGameRequestMove::check_bounds(ledger, *id, dir.0, dir.1, GameBoard::get_bounds_min_for_team(&team), GameBoard::get_bounds_max_for_team(&team)) {
                    return;
                }

                ledger.write::<StatePositionEntities>(|x| {
                    x.positions
                        .insert(*id, (x.positions[id].0 + dir.0, x.positions[id].1 + dir.1));
                });
                ledger.write::<StateEnergy>(|x| {
                    x.all_players
                        .insert(*id, (x.all_players[id].0 - 1, x.all_players[id].1));
                });
            }
            GameEvents::RequestMoveXNeg(id) => {
                if !ECsystemGameRequestMove::check_player_id(ledger, *id) {
                    return;
                }
                if !ECsystemGameRequestMove::check_energy(ledger, *id) {
                    return;
                }
                let Some(team) = ledger.read::<StateTeamAssignments>().team_for(&id) else {
                    return;
                };

                let dir = team.convert_dir(-1, 0);
                if !ECsystemGameRequestMove::check_bounds(ledger, *id, dir.0, dir.1, GameBoard::get_bounds_min_for_team(&team), GameBoard::get_bounds_max_for_team(&team)) {
                    return;
                }

                ledger.write::<StatePositionEntities>(|x| {
                    x.positions
                        .insert(*id, (x.positions[id].0 + dir.0, x.positions[id].1 + dir.1));
                });
                ledger.write::<StateEnergy>(|x| {
                    x.all_players
                        .insert(*id, (x.all_players[id].0 - 1, x.all_players[id].1));
                });
            }
            _ => {}
        }
    }
}
