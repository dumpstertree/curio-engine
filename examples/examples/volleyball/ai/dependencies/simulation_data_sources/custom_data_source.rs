use core::collections::game_state::GameState;
use std::sync::Arc;

use crate::{
    ai::dependencies::simulation_data_source::SimulationDataSource,
    cards::{
        card_attribute_fillers::attribute_filler_ai::CardAttributeFillerAI,
        card_dependencies::{
            builder::{data_dep_filled_for_modifiers::DataDepsFilledForModifiers, filled_attribute_with_permutation::FilledAttributeWithPermutation},
            data_dep_empty::DataDepsEmpty,
        },
        card_instance::CardInstance,
        enums::simulation_manuevers::SimulationManuevers,
    },
    game_board::{Directions, GameBoard},
    state::{
        other::state_terminated::StateTerminated,
        state_ball_mode::{BallModes, StateBallMode},
        state_deck::StateDeck,
        state_energy::StateEnergy,
        state_position_player::StatePositionEntities,
        state_teams::{StateTeamAssignments, Teams},
        state_turn::StateTurn,
    },
};

pub struct CustomDataSource {}
impl SimulationDataSource<(i32, SimulationManuevers), (Teams, Vec<i32>)> for CustomDataSource {
    fn get_cur_user(&self, game_state: &GameState) -> (Teams, Vec<i32>) {
        // get state
        let state_teams = game_state.get::<StateTeamAssignments>();
        let state_turn = game_state.get::<StateTurn>();

        // get team

        let Some(guids) = state_teams
            .team_assignments
            .get(&state_turn.active_instance_id)
        else {
            panic!("Unable to find 'Team' for UID: {}", state_turn.active_instance_id);
        };

        // return
        (state_turn.active_instance_id, guids.clone())
    }
    fn get_all_simulation_actions(&self, game_state: &GameState, user: &(Teams, Vec<i32>)) -> Vec<(i32, SimulationManuevers)> {
        // create the output
        let mut output: Vec<(i32, SimulationManuevers)> = Vec::new();

        // get state terminated
        let state_terminated = game_state.get::<StateTerminated>();

        // this has been marked as terminal so we know there is nothing we can do
        if state_terminated.is_terminated {
            return output;
        }

        // this has been marked as exhuasted so we know there is nothing we can do
        if state_terminated.is_exhuasted {
            return output;
        }

        // get state
        let state_energy = game_state.get::<StateEnergy>();
        let state_pos = game_state.get::<StatePositionEntities>();

        // iterate over each user id on the team
        for user_id in &user.1 {
            // // get the amount of energy this uid has left
            // let Some(energy_for_uid) = state_energy.all_players.get(&user_id) else {
            //     panic!("Failed to find energy for uid: {}", user_id);
            // };

            // append get all manuevers available for this uid
            output.extend(Self::get_available_manuevers(&game_state, &user_id));

            // make sure the ball is not currently being served
            if game_state.get::<StateBallMode>().mode != BallModes::Serve {
                // add end turn make sure this is added before possible breaking from lack of energy
                output.push((*user_id, SimulationManuevers::EndTurn));

                // // if we have enough energy to move add all the directions
                // let has_energy_for_move = energy_for_uid.0 > 0;
                // if has_energy_for_move {
                //     // if we are unable to find a position for this user return the outpue
                //     let Some(pos) = state_pos.positions.get(&user_id) else {
                //         return output;
                //     };

                //     if GameBoard::can_move(&user.0, pos, Directions::Forward) {
                //         output.push((*user_id, SimulationManuevers::MoveEntity(Directions::Forward)));
                //     }
                //     if GameBoard::can_move(&user.0, pos, Directions::Back) {
                //         output.push((*user_id, SimulationManuevers::MoveEntity(Directions::Back)));
                //     }
                //     if GameBoard::can_move(&user.0, pos, Directions::Left) {
                //         output.push((*user_id, SimulationManuevers::MoveEntity(Directions::Left)));
                //     }
                //     if GameBoard::can_move(&user.0, pos, Directions::Right) {
                //         output.push((*user_id, SimulationManuevers::MoveEntity(Directions::Right)));
                //     }
                // }
            }
        }
        // return
        output
    }
}

impl CustomDataSource {
    fn get_available_manuevers_for_cards(game_state: &GameState, uid: &i32, cards: &Vec<Arc<CardInstance>>) -> Vec<(i32, SimulationManuevers)> {
        //
        let mut all_manuevers = Vec::new();
        let state_energy = game_state.get::<StateEnergy>();

        // iterate over each card we were passed in
        for card in cards {
            // get the energy
            let Some(energy_cur_max) = state_energy.all_players.get(uid) else {
                println!("Could not find 'Energy' for UID {}", uid);
                continue;
            };

            // get the cost of this card
            let card_cost = card.get_cost(game_state, *uid);

            // check if we have enough energy to play this card
            let has_energy_to_play = energy_cur_max.0 - card_cost >= 0;
            if !has_energy_to_play {
                continue;
            }

            // check if we can play the cur card  in hand based on gamestate
            let can_play_card = card.has_statement(game_state, uid.clone());
            if !can_play_card {
                continue;
            }

            // the data that stores all the different permutations
            let mut all_data = DataDepsFilledForModifiers::new();

            // iterate over each modifier in the card and populate the list of dependencies
            for modifier in card.get_attributes_modifiers(game_state, uid.clone()) {
                // iterate over each empty dependency for modifier and fill it
                let mut filled = Vec::new();
                for empty in modifier.get_data_dependencies_empty() {
                    match empty {
                        // dependency is a tile - fill the dependency based on type
                        DataDepsEmpty::Tiles(target_type) => {
                            filled.push(CardAttributeFillerAI::fill_dependency_tiles(game_state, uid, target_type));
                        }
                        // dependency is a entity - fill the dependency based on type
                        DataDepsEmpty::Entities(target_type) => {
                            filled.push(CardAttributeFillerAI::fill_dependency_entities(game_state, uid, target_type));
                        }
                        // dependency is a card - fill the dependency based on type
                        DataDepsEmpty::Cards(target_type) => {
                            filled.push(CardAttributeFillerAI::fill_dependency_cards(game_state, uid, target_type));
                        }
                    }
                }

                // add filled
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
                            filled.push(CardAttributeFillerAI::fill_dependency_tiles(game_state, uid, target_type));
                        }
                        // dependency is a entity - fill the dependency based on type
                        DataDepsEmpty::Entities(target_type) => {
                            filled.push(CardAttributeFillerAI::fill_dependency_entities(game_state, uid, target_type));
                        }
                        // dependency is a card - fill the dependency based on type
                        DataDepsEmpty::Cards(target_type) => {
                            filled.push(CardAttributeFillerAI::fill_dependency_cards(game_state, uid, target_type));
                        }
                    }
                }

                // add filled
                all_data.add_modifier_event(FilledAttributeWithPermutation::new(filled));
            }

            // get all the different permutation combinations
            let combined_permutations = all_data.get_data_stack_permutations();

            // convert those permutations into a play
            for combo in combined_permutations {
                all_manuevers.push((*uid, SimulationManuevers::PlayCard(card.clone(), combo)));
            }
        }

        // return
        all_manuevers
    }
    fn get_available_manuevers(game_state: &GameState, user_uid: &i32) -> Vec<(i32, SimulationManuevers)> {
        // create the return object containing all the moves
        let mut all_manuevers = Vec::new();

        // get state
        let state_deck = game_state.get::<StateDeck>();

        // get deck from state
        let Some(deck) = state_deck.deck.get(user_uid) else {
            println!("Failed to find 'Deck' for UID {}", user_uid);
            return all_manuevers;
        };

        // add all the consumable cards
        all_manuevers.append(&mut Self::get_available_manuevers_for_cards(game_state, user_uid, &deck.hand_consumable));

        // add all the persistent cards
        // all_manuevers.append(&mut Self::get_available_manuevers_for_cards(game_state, uid, &deck.hand_persistent));

        // return all
        all_manuevers
    }
}
