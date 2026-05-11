use crate::cards::card_attributes_targets::attribute_target_type_tiles::AttributeTargetTypesTiles;
use crate::cards::card_dependencies::data_dep_filled::DataDepsFilled;
use crate::exploration::exploration_path::RoomTypes;
use crate::game_board::GameBoard;
use crate::state::host::state_exploration::StateExploration;
use crate::state::peer::state_peer_select_targets::SelectStates;
use crate::state::peer::state_peer_select_targets::StatePeerSelectTargets;
use crate::state::state_position_ball::StatePositionBall;
use crate::state::state_teams::StateTeamAssignments;
use curio_core::Vector2Int;
use curio_core::built_in::record::sys_record_input::SysRecordInput;
use curio_core::collections::{event_queue::Nerve, ledger::Ledger};
use curio_core::network_modes::NetworkModes;
use gameplay::context_3d::Context3D;
use gameplay::traits::habit::Habit;
use gameplay::traits::scope::Scope;
use habit::habit;
use std::panic;

#[habit]
pub struct Instance {}
impl Scope for Instance {
    fn is_enabled(&mut self, ledger: &mut Ledger) -> bool {
        ledger
            .read::<StateExploration>()
            .exploration
            .get_cur_room()
            .room_type
            == RoomTypes::Combat
    }
    fn run_on_instance(&mut self, _ledger: &mut Ledger) -> Vec<NetworkModes> {
        NetworkModes::all_peer()
    }
}
impl Habit for Instance {
    fn enable(&mut self, _: &mut Ledger, _: &mut Context3D, _: &mut Nerve) {}
    fn tick(&mut self, ledger: &mut Ledger, _: &mut Context3D, _events: &mut Nerve) {
        let state_select_targets = ledger.read::<StatePeerSelectTargets>();
        let state_input = ledger.read::<SysRecordInput>();
        // mode is currently set to NONE
        let Some(mode) = &state_select_targets.enabled else {
            return;
        };

        match mode {
            SelectStates::Enabled(target, _working_state) => {
                let state_team = ledger.read::<StateTeamAssignments>();
                let team = state_team.team_for(&ledger.network.me().guid).unwrap();
                let mut all_targets = self.get_all_tiles(ledger, *target);

                if all_targets.len() == 0 {
                    panic!("invalid number of targets!");
                }

                match target {
                    AttributeTargetTypesTiles::SelectInRangeLocalToBall(_, _) => {
                        // get the state of input
                        let input_submit = state_input.mapped[0].get_button_or_default("turn_end");
                        let input_fwd = state_input.mapped[0].get_button_or_default("move_forward");
                        let input_back = state_input.mapped[0].get_button_or_default("move_back");
                        let input_left = state_input.mapped[0].get_button_or_default("move_left");
                        let input_right = state_input.mapped[0].get_button_or_default("move_right");

                        // clamp to a tile in our range
                        if !all_targets.contains(&state_select_targets.selected_index) {
                            ledger.write::<StatePeerSelectTargets>(|x| {
                                x.selected_index = all_targets[0];
                            });
                        }
                        // get submition event
                        if input_submit.went_up {
                            ledger.write::<StatePeerSelectTargets>(|x| x.enabled = Some(SelectStates::Completed(DataDepsFilled::Tiles(vec![x.selected_index]))));
                            println!("submit");
                        }

                        match team {
                            crate::state::state_teams::Teams::Red => {
                                //edit index -> fwd
                                if input_fwd.went_up {
                                    all_targets.retain(|x| x.y > state_select_targets.selected_index.y);
                                    if all_targets.len() == 0 {
                                        return;
                                    }
                                }
                                // edit index -> back
                                if input_back.went_up {
                                    all_targets.retain(|x| x.y < state_select_targets.selected_index.y);
                                    if all_targets.len() == 0 {
                                        return;
                                    }
                                }
                                // edit index -> left
                                if input_left.went_up {
                                    all_targets.retain(|x| x.x < state_select_targets.selected_index.x);
                                    if all_targets.len() == 0 {
                                        return;
                                    }
                                }
                                // edit index -> right
                                if input_right.went_up {
                                    all_targets.retain(|x| x.x > state_select_targets.selected_index.x);
                                    if all_targets.len() == 0 {
                                        return;
                                    }
                                }
                            }
                            crate::state::state_teams::Teams::Blue => {
                                //edit index -> fwd
                                if input_fwd.went_up {
                                    all_targets.retain(|x| x.y < state_select_targets.selected_index.y);
                                    if all_targets.len() == 0 {
                                        return;
                                    }
                                }
                                // edit index -> back
                                if input_back.went_up {
                                    all_targets.retain(|x| x.y > state_select_targets.selected_index.y);
                                    if all_targets.len() == 0 {
                                        return;
                                    }
                                }
                                // edit index -> left
                                if input_left.went_up {
                                    all_targets.retain(|x| x.x > state_select_targets.selected_index.x);
                                    if all_targets.len() == 0 {
                                        return;
                                    }
                                }
                                // edit index -> right
                                if input_right.went_up {
                                    all_targets.retain(|x| x.x < state_select_targets.selected_index.x);
                                    if all_targets.len() == 0 {
                                        return;
                                    }
                                }
                            }
                        }

                        let mut dist = 9999;
                        let mut closest = Vector2Int::zero();
                        for t in all_targets {
                            let d = state_select_targets.selected_index - t;
                            let d = d.x.abs() + d.y.abs();
                            if d <= dist {
                                dist = d;
                                closest = t;
                            }
                        }
                        ledger.write::<StatePeerSelectTargets>(|x| {
                            x.selected_index = closest;
                            println!("try set {}", closest);
                        });
                    }

                    AttributeTargetTypesTiles::SelectOpponentBackCorner | AttributeTargetTypesTiles::SelectAny | AttributeTargetTypesTiles::SelectOnTeamUser | AttributeTargetTypesTiles::SelectOnTeamOpponent => {
                        // get the state of input
                        let input_submit = state_input.mapped[0].get_button_or_default("turn_end");
                        let input_fwd = state_input.mapped[0].get_button_or_default("move_forward");
                        let input_back = state_input.mapped[0].get_button_or_default("move_back");
                        let input_left = state_input.mapped[0].get_button_or_default("move_left");
                        let input_right = state_input.mapped[0].get_button_or_default("move_right");

                        // clamp to a tile in our range
                        if !all_targets.contains(&state_select_targets.selected_index) {
                            ledger.write::<StatePeerSelectTargets>(|x| {
                                x.selected_index = all_targets[0];
                            });
                        }
                        // get submition event
                        if input_submit.went_up {
                            ledger.write::<StatePeerSelectTargets>(|x| x.enabled = Some(SelectStates::Completed(DataDepsFilled::Tiles(vec![x.selected_index]))));
                            println!("submit");
                        }

                        match team {
                            crate::state::state_teams::Teams::Red => {
                                //edit index -> fwd
                                if input_fwd.went_up {
                                    all_targets.retain(|x| x.y > state_select_targets.selected_index.y);
                                    if all_targets.len() == 0 {
                                        return;
                                    }
                                }
                                // edit index -> back
                                if input_back.went_up {
                                    all_targets.retain(|x| x.y < state_select_targets.selected_index.y);
                                    if all_targets.len() == 0 {
                                        return;
                                    }
                                }
                                // edit index -> left
                                if input_left.went_up {
                                    all_targets.retain(|x| x.x < state_select_targets.selected_index.x);
                                    if all_targets.len() == 0 {
                                        return;
                                    }
                                }
                                // edit index -> right
                                if input_right.went_up {
                                    all_targets.retain(|x| x.x > state_select_targets.selected_index.x);
                                    if all_targets.len() == 0 {
                                        return;
                                    }
                                }
                            }
                            crate::state::state_teams::Teams::Blue => {
                                //edit index -> fwd
                                if input_fwd.went_up {
                                    all_targets.retain(|x| x.y < state_select_targets.selected_index.y);
                                    if all_targets.len() == 0 {
                                        return;
                                    }
                                }
                                // edit index -> back
                                if input_back.went_up {
                                    all_targets.retain(|x| x.y > state_select_targets.selected_index.y);
                                    if all_targets.len() == 0 {
                                        return;
                                    }
                                }
                                // edit index -> left
                                if input_left.went_up {
                                    all_targets.retain(|x| x.x > state_select_targets.selected_index.x);
                                    if all_targets.len() == 0 {
                                        return;
                                    }
                                }
                                // edit index -> right
                                if input_right.went_up {
                                    all_targets.retain(|x| x.x < state_select_targets.selected_index.x);
                                    if all_targets.len() == 0 {
                                        return;
                                    }
                                }
                            }
                        }

                        let mut dist = 9999;
                        let mut closest = Vector2Int::zero();
                        for t in all_targets {
                            let d = state_select_targets.selected_index - t;
                            let d = d.x.abs() + d.y.abs();
                            if d <= dist {
                                dist = d;
                                closest = t;
                            }
                        }
                        ledger.write::<StatePeerSelectTargets>(|x| {
                            x.selected_index = closest;
                            println!("try set {}", closest);
                        });
                    }
                    _ => {
                        println!("Trying to select tile with inccorect target type");
                    }
                }
            }
            _ => {}
        }
    }
}
impl Instance {
    fn get_all_tiles(&self, ledger: &Ledger, target_type: AttributeTargetTypesTiles) -> Vec<Vector2Int> {
        match target_type {
            AttributeTargetTypesTiles::SelectOpponentBackCorner => {
                let user_uid = ledger.network.me().guid;
                let team = ledger
                    .read::<StateTeamAssignments>()
                    .team_for(&user_uid)
                    .unwrap();
                return GameBoard::get_back_corners_for_team(&team.next_team());
            }

            AttributeTargetTypesTiles::SelectInRangeLocalToBall(min, max) => {
                let pos_ball = ledger.read::<StatePositionBall>();
                let user_uid = ledger.network.me().guid;
                let team = ledger
                    .read::<StateTeamAssignments>()
                    .team_for(&user_uid)
                    .unwrap();

                let mut targets = Vec::new();
                for x in min.x..(max.x + 1) {
                    for y in min.y..(max.y + 1) {
                        let c = team.convert_dir(x, y);
                        targets.push(Vector2Int::new(pos_ball.column + c.0, pos_ball.row + c.1));
                    }
                }

                // println!("Faile d unwrap");
                return targets;
            }
            AttributeTargetTypesTiles::SelectAny => GameBoard::get_tiles(),
            AttributeTargetTypesTiles::SelectOnTeamUser => {
                let user_uid = ledger.network.me().guid;
                let team = ledger.read::<StateTeamAssignments>().team_for(&user_uid);
                if let Some(team) = team {
                    return GameBoard::get_tiles_for_team(&team);
                }
                println!("Failed unwrap");
                return Vec::new();
            }
            AttributeTargetTypesTiles::SelectOnTeamOpponent => {
                let user_uid = ledger.network.me().guid;
                let team = ledger.read::<StateTeamAssignments>().team_for(&user_uid);
                if let Some(team) = team {
                    return GameBoard::get_tiles_for_team(&team.next_team());
                }
                println!("Failed unwrap");
                return Vec::new();
            }
            _ => {
                println!("Invalid Selection Type");
                return Vec::new();
            }
        }
    }
}
