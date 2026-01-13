use crate::cards::card_attributes_targets::attribute_target_type_tiles::AttributeTargetTypesTiles;
use crate::cards::card_dependencies::data_dep_filled::DataDepsFilled;
use crate::exploration::exploration_path::RoomTypes;
use crate::game_board::GameBoard;
use crate::state::host::state_exploration::StateExploration;
use crate::state::peer::state_peer_select_targets::SelectStates;
use crate::state::peer::state_peer_select_targets::StatePeerSelectTargets;
use crate::state::state_teams::StateTeamAssignments;
use built_in_state::state_input::InputState;
use core::collections::vector2_int::Vector2Int;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_system::habit;
use std::panic;
use system_component_default_gameplay::traits::habit::Habit;
use system_component_default_gameplay::traits::scope::Scope;
use system_component_default_gameplay::context_3d::Context3D;

#[habit]
pub struct Instance {}
impl Scope for Instance {
    fn is_enabled(&mut self, game_state: &mut GameState) -> bool {
        game_state
            .get::<StateExploration>()
            .exploration
            .get_cur_room()
            .room_type
            == RoomTypes::Combat
    }
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all_peer()
    }
}
impl Habit for Instance {
    fn enable(&mut self, _: &mut GameState, _: &mut Context3D, _: &mut EventQueue) {}
    fn tick(&mut self, game_state: &mut GameState, _: &mut Context3D, events: &mut EventQueue) {
        let state_select_targets = game_state.get::<StatePeerSelectTargets>();
        let state_input = game_state.get::<InputState>();
        // mode is currently set to NONE
        let Some(mode) = state_select_targets.enabled else {
            return;
        };

        match mode {
            SelectStates::Enabled(target, working_state) => {
                let state_team = game_state.get::<StateTeamAssignments>();
                let team = state_team.team_for(&game_state.instance_id).unwrap();
                let all_targets = self.get_all_tiles(game_state, target);

                if all_targets.len() == 0 {
                    panic!("invalid number of targets!");
                }

                match target {
                    AttributeTargetTypesTiles::SelectAny | AttributeTargetTypesTiles::SelectOnTeamUser | AttributeTargetTypesTiles::SelectOnTeamOpponent => {
                        // get the state of input
                        let input_submit = state_input.mapped[0].get_button_or_default("turn_end");
                        let input_fwd = state_input.mapped[0].get_button_or_default("move_forward");
                        let input_back = state_input.mapped[0].get_button_or_default("move_back");
                        let input_left = state_input.mapped[0].get_button_or_default("move_left");
                        let input_right = state_input.mapped[0].get_button_or_default("move_right");

                        // clamp to a tile in our range
                        if !all_targets.contains(&state_select_targets.selected_index) {
                            game_state.edit::<StatePeerSelectTargets>(|x| {
                                x.selected_index = all_targets[0];
                            });
                        }
                        // get submition event
                        if input_submit.went_up {
                            game_state.edit::<StatePeerSelectTargets>(|x| x.enabled = Some(SelectStates::Completed(DataDepsFilled::Tiles(vec![x.selected_index]))));
                            println!("submit");
                        }

                        //edit index -> fwd
                        if input_fwd.went_up {
                            let c = team.convert_dir(0, 1);
                            let new_index = state_select_targets.selected_index + Vector2Int::new(c.0, c.1);
                            if all_targets.contains(&new_index) {
                                game_state.edit::<StatePeerSelectTargets>(|x| {
                                    x.selected_index = new_index;
                                });
                            }
                        }
                        // edit index -> back
                        if input_back.went_up {
                            let c = team.convert_dir(0, -1);
                            let new_index = state_select_targets.selected_index + Vector2Int::new(c.0, c.1);
                            if all_targets.contains(&new_index) {
                                game_state.edit::<StatePeerSelectTargets>(|x| {
                                    x.selected_index = new_index;
                                });
                            }
                        }
                        // edit index -> left
                        if input_left.went_up {
                            let c = team.convert_dir(-1, 0);
                            let new_index = state_select_targets.selected_index + Vector2Int::new(c.0, c.1);
                            if all_targets.contains(&new_index) {
                                game_state.edit::<StatePeerSelectTargets>(|x| {
                                    x.selected_index = new_index;
                                });
                            }
                        }
                        // edit index -> right
                        if input_right.went_up {
                            let c = team.convert_dir(1, 0);
                            let new_index = state_select_targets.selected_index + Vector2Int::new(c.0, c.1);
                            if all_targets.contains(&new_index) {
                                game_state.edit::<StatePeerSelectTargets>(|x| {
                                    x.selected_index = new_index;
                                });
                            }
                        }
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
    fn get_all_tiles(&self, game_state: &GameState, target_type: AttributeTargetTypesTiles) -> Vec<Vector2Int> {
        match target_type {
            AttributeTargetTypesTiles::SelectAny => GameBoard::get_tiles(),
            AttributeTargetTypesTiles::SelectOnTeamUser => {
                let user_uid = game_state.instance_id;
                let team = game_state.get::<StateTeamAssignments>().team_for(&user_uid);
                if let Some(team) = team {
                    return GameBoard::get_tiles_for_team(&team);
                }
                println!("Failed unwrap");
                return Vec::new();
            }
            AttributeTargetTypesTiles::SelectOnTeamOpponent => {
                let user_uid = game_state.instance_id;
                let team = game_state.get::<StateTeamAssignments>().team_for(&user_uid);
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
