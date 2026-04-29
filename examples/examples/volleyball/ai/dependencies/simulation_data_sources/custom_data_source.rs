use curio_core::collections::ledger::Ledger;
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
    state::{
        other::state_terminated::StateTerminated,
        state_ball_mode::{BallModes, StateBallMode},
        state_deck::StateDeck,
        state_energy::StateEnergy,
        state_teams::{StateTeamAssignments, Teams},
        state_turn::StateTurn,
    },
};

pub struct CustomDataSource {}
impl SimulationDataSource<(i32, SimulationManuevers), (Teams, Vec<i32>)> for CustomDataSource {
    fn get_cur_user(&self, ledger: &Ledger) -> (Teams, Vec<i32>) {
        // get state
        let state_teams = ledger.read::<StateTeamAssignments>();
        let state_turn = ledger.read::<StateTurn>();

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
    fn get_all_simulation_actions(&self, ledger: &Ledger, user: &(Teams, Vec<i32>)) -> Vec<(i32, SimulationManuevers)> {
        // create the output
        let mut output: Vec<(i32, SimulationManuevers)> = Vec::new();

        // get state terminated
        let state_terminated = ledger.read::<StateTerminated>();

        // this has been marked as terminal so we know there is nothing we can do
        if state_terminated.is_terminated {
            return output;
        }

        // this has been marked as exhuasted so we know there is nothing we can do
        if state_terminated.is_exhuasted {
            return output;
        }

        // iterate over each user id on the team
        for user_id in &user.1 {
            // append get all manuevers available for this uid
            output.extend(Self::get_available_manuevers(&ledger, &user_id));

            // make sure the ball is not currently being served
            if ledger.read::<StateBallMode>().mode != BallModes::Serve {
                // add end turn make sure this is added before possible breaking from lack of energy
                output.push((*user_id, SimulationManuevers::EndTurn));
            }
        }
        // return
        output
    }
}

impl CustomDataSource {
    fn get_available_manuevers_for_cards(ledger: &Ledger, uid: &i32, cards: &Vec<Arc<CardInstance>>) -> Vec<(i32, SimulationManuevers)> {
        //
        let mut all_manuevers = Vec::new();
        let state_energy = ledger.read::<StateEnergy>();

        // iterate over each card we were passed in
        for card in cards {
            // get the energy
            let Some(energy_cur_max) = state_energy.all_players.get(uid) else {
                println!("Could not find 'Energy' for UID {}", uid);
                continue;
            };

            // get the cost of this card
            let card_cost = card.get_cost(ledger, *uid);

            // check if we have enough energy to play this card
            let has_energy_to_play = energy_cur_max.0 - card_cost >= 0;
            if !has_energy_to_play {
                continue;
            }

            // check if we can play the cur card  in hand based on gamestate
            let can_play_card = card.has_statement(ledger, uid.clone());
            if !can_play_card {
                continue;
            }

            // the data that stores all the different permutations
            let mut all_data = DataDepsFilledForModifiers::new();

            // iterate over each modifier in the card and populate the list of dependencies
            for modifier in card.get_attributes_modifiers(ledger, uid.clone()) {
                // iterate over each empty dependency for modifier and fill it
                let mut filled = Vec::new();
                for empty in modifier.get_data_dependencies_empty() {
                    match empty {
                        // dependency is a tile - fill the dependency based on type
                        DataDepsEmpty::Tiles(target_type) => {
                            filled.push(CardAttributeFillerAI::fill_dependency_tiles(ledger, uid, target_type));
                        }
                        // dependency is a entity - fill the dependency based on type
                        DataDepsEmpty::Entities(target_type) => {
                            filled.push(CardAttributeFillerAI::fill_dependency_entities(ledger, uid, target_type));
                        }
                        // dependency is a card - fill the dependency based on type
                        DataDepsEmpty::Cards(target_type) => {
                            filled.push(CardAttributeFillerAI::fill_dependency_cards(ledger, uid, target_type));
                        }
                    }
                }

                // add filled
                all_data.add_modifier_atts(FilledAttributeWithPermutation::new(filled));
            }
            // iterate over each event in the card and populate the list of dependencies
            for event in card.get_attributes_events(ledger, uid.clone()) {
                // iterate over each empty dependency for modifier and fill it
                let mut filled = Vec::new();
                for empty in event.get_data_dependencies_empty() {
                    match empty {
                        // dependency is a tile - fill the dependency based on type
                        DataDepsEmpty::Tiles(target_type) => {
                            filled.push(CardAttributeFillerAI::fill_dependency_tiles(ledger, uid, target_type));
                        }
                        // dependency is a entity - fill the dependency based on type
                        DataDepsEmpty::Entities(target_type) => {
                            filled.push(CardAttributeFillerAI::fill_dependency_entities(ledger, uid, target_type));
                        }
                        // dependency is a card - fill the dependency based on type
                        DataDepsEmpty::Cards(target_type) => {
                            filled.push(CardAttributeFillerAI::fill_dependency_cards(ledger, uid, target_type));
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
    fn get_available_manuevers(ledger: &Ledger, user_uid: &i32) -> Vec<(i32, SimulationManuevers)> {
        // create the return object containing all the moves
        let mut all_manuevers = Vec::new();

        // get state
        let state_deck = ledger.read::<StateDeck>();

        // get deck from state
        let Some(deck) = state_deck.deck.get(user_uid) else {
            println!("Failed to find 'Deck' for UID {}", user_uid);
            return all_manuevers;
        };

        // add all the consumable cards
        all_manuevers.append(&mut Self::get_available_manuevers_for_cards(ledger, user_uid, &deck.hand_consumable));

        // add all the persistent cards
        // all_manuevers.append(&mut Self::get_available_manuevers_for_cards(ledger, uid, &deck.hand_persistent));

        // return all
        all_manuevers
    }
}
