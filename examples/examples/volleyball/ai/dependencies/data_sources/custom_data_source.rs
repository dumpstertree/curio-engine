use core::{
    collections::{game_state::GameState, vector2_int::Vector2Int},
    random::Random,
};
use std::sync::Arc;

use crate::{
    ai::{StateTerminated::StateTerminated, dependencies::data_source::SimulationDataSource},
    ai_resolver::{DataDepsFilledAllPermutations, DataDepsFilledForModifiers, Directions, FilledAttributeWithPermutation, Move},
    cards::{attribute_target_type_entities::AttribtuteTargetTypesEntities, attribute_target_type_tiles::AttributeTargetTypesTiles, data_dep_empty::DataDepsEmpty, data_dep_filled::DataDepsFilled},
    game_board::GameBoard,
    state::{
        state_ball_mode::{BallModes, StateBallMode},
        state_deck::StateDeck,
        state_energy::StateEnergy,
        state_position_ball::StatePositionBall,
        state_position_player::StatePositionPlayer,
        state_teams::{StateTeamAssignments, Teams},
        state_turn::StateTurn,
    },
};

pub struct CustomDataSource {}
impl SimulationDataSource<Move, (Teams, i32)> for CustomDataSource {
    fn get_cur_user(&self, game_state: &GameState) -> (Teams, i32) {
        // get state
        let state_teams = game_state.get::<StateTeamAssignments>();
        let state_turn = game_state.get::<StateTurn>();

        // get team
        let Some(team) = state_teams.team_for(&state_turn.active_instance_id) else {
            panic!("");
        };

        // return
        (team, state_turn.active_instance_id)
    }

    fn all(&self, game_state: &GameState, user: &(Teams, i32)) -> Vec<Move> {
        // create the output
        let mut output = Vec::new();

        // this has been marked as terminal so we know there is nothing we can do
        if game_state.get::<StateTerminated>().is_terminated {
            return output;
        }
        if game_state.get::<StateTerminated>().is_exhuasted {
            return output;
        }

        // get state
        let state_energy = game_state.get::<StateEnergy>();
        let state_pos = game_state.get::<StatePositionPlayer>();

        // get the amount of energy this uid has left
        let Some(energy_for_uid) = state_energy.all_players.get(&user.1) else {
            panic!("Failed to find energy for uid: {}", user.1);
        };

        // append get all manuevers available for this uid
        output.extend(Self::get_available_manuevers(&game_state, &user.1));

        // make sure the ball is not currently being served
        if game_state.get::<StateBallMode>().mode != BallModes::Serve {
            // add end turn make sure this is added before possible breaking from lack of energy
            output.push(Move::EndTurn);

            // if we have enough energy to move add all the directions
            let has_energy_for_move = energy_for_uid.0 > 0;
            if has_energy_for_move {
                // if we are unable to find a position for this user return the outpue
                let Some(pos) = state_pos.positions.get(&user.1) else {
                    return output;
                };

                if GameBoard::can_move(&user.0, pos, Directions::Forward) {
                    output.push(Move::Move(Directions::Forward));
                }
                if GameBoard::can_move(&user.0, pos, Directions::Back) {
                    output.push(Move::Move(Directions::Back));
                }
                if GameBoard::can_move(&user.0, pos, Directions::Left) {
                    output.push(Move::Move(Directions::Left));
                }
                if GameBoard::can_move(&user.0, pos, Directions::Right) {
                    output.push(Move::Move(Directions::Right));
                }
            }
        }

        // return
        output
    }
}
impl CustomDataSource {
    fn fill_dependency_tiles(game_state: &GameState, uid: &i32, empty: AttributeTargetTypesTiles) -> DataDepsFilledAllPermutations {
        // create the list of permutations
        let mut permuatations = DataDepsFilledAllPermutations::new();

        // match for empty type
        match empty {
            AttributeTargetTypesTiles::RandomOnTeamUser => {
                // get state
                let state_team = game_state.get::<StateTeamAssignments>();

                // get the team for this user
                let Some(my_team) = state_team.team_for(&uid) else {
                    println!("Failed to find team for uid: {}", uid);
                    return permuatations;
                };

                let min = GameBoard::get_bounds_min(&my_team);
                let max = GameBoard::get_bounds_max(&my_team);

                let random_x = Random::range_int(min.x, max.x);
                let random_z = Random::range_int(min.y, max.y);

                permuatations.add_permutation(DataDepsFilled::Tiles(vec![Vector2Int::new(random_x, random_z)]));
            }
            AttributeTargetTypesTiles::RandomOnTeamOpponent => {
                // get state
                let state_team = game_state.get::<StateTeamAssignments>();

                // get the team for this user
                let Some(my_team) = state_team.team_for(&uid) else {
                    println!("Failed to find team for uid: {}", uid);
                    return permuatations;
                };

                let other_team = my_team.next_team();

                let min = GameBoard::get_bounds_min(&other_team);
                let max = GameBoard::get_bounds_max(&other_team);

                let random_x = Random::range_int(min.x, max.x);
                let random_z = Random::range_int(min.y, max.y);

                permuatations.add_permutation(DataDepsFilled::Tiles(vec![Vector2Int::new(random_x, random_z)]));
            }
            AttributeTargetTypesTiles::RandomInRangeLocal(min, max) => {
                let state_turn = game_state.get::<StateTurn>();
                let state_teams = game_state.get::<StateTeamAssignments>();
                let state_position_ball = game_state.get::<StatePositionBall>();

                let team = state_teams
                    .team_for(&state_turn.active_instance_id)
                    .unwrap();
                let min = team.convert_dir(min.x, min.y);
                let max = team.convert_dir(max.x, max.y);
                let random_x = Random::range_int(min.0, max.0);
                let random_z = Random::range_int(min.1, max.1);

                let col = state_position_ball.column + random_x;
                let row = state_position_ball.row + random_z;

                permuatations.add_permutation(DataDepsFilled::Tiles(vec![Vector2Int::new(col, row)]));
            }
            AttributeTargetTypesTiles::RandomInRangeGlobal(min, max) => {
                let random_x = Random::range_int(min.x, max.x);
                let random_z = Random::range_int(min.y, max.y);

                permuatations.add_permutation(DataDepsFilled::Tiles(vec![Vector2Int::new(random_x, random_z)]));
            }
            _ => {}
        }

        // return the now filled permutations
        permuatations
    }
    fn fill_dependency_entities(game_state: &GameState, uid: &i32, empty: AttribtuteTargetTypesEntities) -> DataDepsFilledAllPermutations {
        // create the list of permutations
        let mut permuatations = DataDepsFilledAllPermutations::new();

        match empty {
            AttribtuteTargetTypesEntities::User => {
                // add a permutation of your uid
                permuatations.add_permutation(DataDepsFilled::Entities(vec![uid.clone()]))
            }
            AttribtuteTargetTypesEntities::Select => {
                // get state
                let state_team = game_state.get::<StateTeamAssignments>();

                // iterate over each team + uids
                for team_ids in state_team.team_assignments {
                    // iterate over each uid on each team
                    for uid in team_ids.1 {
                        // "select" each uid as a different permutation
                        permuatations.add_permutation(DataDepsFilled::Entities(vec![uid]));
                    }
                }
            }
            AttribtuteTargetTypesEntities::RandomAny => {
                // get state
                let state_team = game_state.get::<StateTeamAssignments>();

                // get the uids of a random team
                let Some(random_team) = state_team.team_assignments.get(&Teams::random()) else {
                    println!("Failed to get random team");
                    return permuatations;
                };

                // roll an index between 0 and the max num of users on the team
                let random_user_index = Random::range_int(0, random_team.len() as i32);

                // get the user for the rolled index
                let Some(random_user_id) = random_team.get(random_user_index as usize) else {
                    println!("Failed to get random user");
                    return permuatations;
                };

                // add a permutation for random
                permuatations.add_permutation(DataDepsFilled::Entities(vec![*random_user_id]));
            }
            AttribtuteTargetTypesEntities::RandomOpponent => {
                // get state
                let state_team = game_state.get::<StateTeamAssignments>();

                // get the team for this user
                let Some(my_team) = state_team.team_for(&uid) else {
                    println!("Failed to find team for uid: {}", uid);
                    return permuatations;
                };

                // get the opposing team based on our team
                let opponent_team = my_team.next_team();

                // get all uids on opponent team
                let Some(opponent_uids) = state_team.team_assignments.get(&opponent_team) else {
                    println!("Failed to find uids for team: {}", opponent_team);
                    return permuatations;
                };

                // roll an index between 0 and the max num of users on the team
                let random_user_index = Random::range_int(0, opponent_uids.len() as i32);

                // get the user for the rolled index
                let Some(random_user_id) = opponent_uids.get(random_user_index as usize) else {
                    println!("Failed to get random user for index: {}", random_user_index);
                    return permuatations;
                };

                // add a permutation for random
                permuatations.add_permutation(DataDepsFilled::Entities(vec![*random_user_id]));
            }
        }

        // return the now filled permutations
        permuatations
    }
    fn get_available_manuevers(game_state: &GameState, uid: &i32) -> Vec<Move> {
        // create the return object containing all the moves
        let mut all_manuevers = Vec::new();

        // get state
        let state_deck = game_state.get::<StateDeck>();

        // get deck from state
        let Some(deck) = state_deck.deck.get(uid) else {
            return all_manuevers;
        };

        // iterate over each card in hand
        for card in &deck.hand_consumable {
            let e = game_state
                .get::<StateEnergy>()
                .all_players
                .get(uid)
                .unwrap()
                .0;
            if e - card.get_cost(game_state, *uid) < 0 {
                continue;
            }

            // the data that stores all the different permutations
            let mut all_data = DataDepsFilledForModifiers::new();

            // check if we can play the cur card  in hand based on gamestate
            let can_play_card = card.has_statement(game_state, uid.clone());
            if !can_play_card {
                continue;
            }

            // iterate over each modifier in the card and populate the list of dependencies
            for modifier in card.get_attributes_modifiers(game_state, uid.clone()) {
                // iterate over each empty dependency for modifier and fill it
                let mut filled = Vec::new();
                for empty in modifier.get_data_dependencies_empty() {
                    match empty {
                        // dependency is a tile - fill the dependency based on type
                        DataDepsEmpty::Tiles(target_type) => {
                            filled.push(Self::fill_dependency_tiles(game_state, uid, target_type));
                        }

                        // dependency is a entity - fill the dependency based on type
                        DataDepsEmpty::Entities(target_type) => {
                            filled.push(Self::fill_dependency_entities(game_state, uid, target_type));
                        }
                        _ => {}
                    }
                }
                all_data.add_modifier_atts(FilledAttributeWithPermutation::new(filled));
            }
            // iterate over each event in the card and populate the list of dependencies
            for event in card.get_attributes_events(game_state, uid.clone()) {
                // iterate over each empty dependency for modifier and fill it
                let mut filled = Vec::new();
                for empty in event.get_data_dependencies_empty() {
                    match empty {
                        // dependency is a tile - fill the dependency based on type
                        DataDepsEmpty::Tiles(target_type) => {
                            filled.push(Self::fill_dependency_tiles(game_state, uid, target_type));
                        }

                        // dependency is a entity - fill the dependency based on type
                        DataDepsEmpty::Entities(target_type) => {
                            filled.push(Self::fill_dependency_entities(game_state, uid, target_type));
                        }
                        _ => {}
                    }
                }
                all_data.add_modifier_event(FilledAttributeWithPermutation::new(filled));
            }

            // if we didnt end up filling anything in dependencies break early
            // let has_filled_dependencies = !all_data.modifiers_atts.is_empty() && !all_data.modifiers_events.is_empty();
            // if has_filled_dependencies {
            // get all the different permutation combinations
            let combined_permutations = all_data.get_data_stack_permutations();

            // convert those permutations into a play
            for combo in combined_permutations {
                all_manuevers.push(Move::Play(Arc::clone(card), combo));
            }
            // } else {
            //     // No modifier targets — still produce a base move
            //     all_manuevers.push(Move::Play(Arc::clone(card), FilledCardResponse::new(vec![], vec![])));
            // }
        }

        // iterate over each card in hand
        for card in &deck.hand_persistent {
            if card.card_id == "rest" {
                println!(
                    "card id: {}, evnt atts{}, mod atts{}",
                    card.card_id,
                    card.get_attributes_events(game_state, game_state.instance_id)
                        .len(),
                    card.get_attributes_modifiers(game_state, game_state.instance_id)
                        .len()
                );

                panic!("");
            }

            // the data that stores all the different permutations
            let mut all_data = DataDepsFilledForModifiers::new();

            // check if we can play the cur card  in hand based on gamestate
            let can_play_card = card.has_statement(game_state, uid.clone());
            if !can_play_card {
                continue;
            }

            // iterate over each modifier in the card and populate the list of dependencies
            for modifier in card.get_attributes_modifiers(game_state, uid.clone()) {
                // iterate over each empty dependency for modifier and fill it
                let mut filled = Vec::new();
                for empty in modifier.get_data_dependencies_empty() {
                    match empty {
                        // dependency is a tile - fill the dependency based on type
                        DataDepsEmpty::Tiles(target_type) => {
                            filled.push(Self::fill_dependency_tiles(game_state, uid, target_type));
                        }

                        // dependency is a entity - fill the dependency based on type
                        DataDepsEmpty::Entities(target_type) => {
                            filled.push(Self::fill_dependency_entities(game_state, uid, target_type));
                        }
                        _ => {}
                    }
                }
                all_data.add_modifier_atts(FilledAttributeWithPermutation::new(filled));
            }
            // iterate over each event in the card and populate the list of dependencies
            for event in card.get_attributes_events(game_state, uid.clone()) {
                // iterate over each empty dependency for modifier and fill it
                let mut filled = Vec::new();
                for empty in event.get_data_dependencies_empty() {
                    match empty {
                        // dependency is a tile - fill the dependency based on type
                        DataDepsEmpty::Tiles(target_type) => {
                            filled.push(Self::fill_dependency_tiles(game_state, uid, target_type));
                        }

                        // dependency is a entity - fill the dependency based on type
                        DataDepsEmpty::Entities(target_type) => {
                            filled.push(Self::fill_dependency_entities(game_state, uid, target_type));
                        }
                        _ => {}
                    }
                }
                all_data.add_modifier_event(FilledAttributeWithPermutation::new(filled));
            }

            if card.card_id == "serve" {
                // println!("num event atts {}", all_data.modifiers_events.len());
                // println!("num event deps {}", all_data.modifiers_events[1].filled.len());
            }
            // if we didnt end up filling anything in dependencies break early
            // let has_filled_dependencies = !all_data.modifiers_atts.is_empty() || !all_data.modifiers_events.is_empty();
            // if has_filled_dependencies {
            //     // get all the different permutation combinations
            let combined_permutations = all_data.get_data_stack_permutations();

            // if card.card_id == "serve" {
            //     println!("combined num event atts {}", combined_permutations[0].event.len());
            //     println!("combined num event deps {}", combined_permutations[0].event[1].filled.len());
            // }
            // convert those permutations into a play
            for combo in combined_permutations {
                all_manuevers.push(Move::Play(Arc::clone(card), combo));
            }
            // } else {
            //     // No modifier targets — still produce a base move
            //     all_manuevers.push(Move::Play(Arc::clone(card), FilledCardResponse::new(vec![], vec![])));
            // }
        }

        // return all
        all_manuevers
    }
}
