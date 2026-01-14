use curio_core::collections::game_state::GameState;
use std::sync::Arc;

use crate::{
    ai::dependencies::simulation_delegate::SimulationDelegate,
    cards::{
        card_dependencies::filled_card_response::FilledCardResponse,
        card_event_runner::CardEventRunner,
        card_instance::CardInstance,
        enums::{attribute_clear_flag::ModifierClearFlag, simulation_manuevers::SimulationManuevers},
    },
    game_board::{Directions, GameBoard},
    state::{other::state_terminated::StateTerminated, state_energy::StateEnergy, state_position_player::StatePositionEntities, state_teams::Teams},
};
pub struct CustomDelegate {}
impl SimulationDelegate<(i32, SimulationManuevers), (Teams, Vec<i32>)> for CustomDelegate {
    // simulates the current move into the gamestate
    fn simulate(&self, game_state: &mut GameState, user: &(Teams, Vec<i32>), manuever: &(i32, SimulationManuevers)) {
        let user_team = user.0;
        let user_id = manuever.0;
        let manuever = manuever.1.clone();
        // each manuever type is handled differently. call the corresponding fn for each manuever
        match manuever {
            SimulationManuevers::PlayCard(card, data) => Self::make_manuever_card(game_state, &data, &card, &(user_team, user_id)),
            SimulationManuevers::MoveEntity(vector2_int) => Self::make_manuever_move(game_state, &vector2_int, &(user_team, user_id)),
            SimulationManuevers::EndTurn => Self::make_manuever_end(game_state),
            SimulationManuevers::Invalid => {}
        }

        // get state
        let state_energy = game_state.get::<StateEnergy>();

        // get the energy for this user
        let Some(energy_cur_max) = state_energy.all_players.get(&user_id) else {
            println!("Could not find 'Energy' for UID: {}", user_id);
            return;
        };

        // if we have 0 or less energy we are exhuasted which is a more specific typer of terminated
        let is_exhuasted = energy_cur_max.0 <= 0;
        if is_exhuasted {
            // edit gamestate for represent our exhausted state
            game_state.edit::<StateTerminated>(|x| {
                x.is_terminated = true;
                x.is_exhuasted = true;
            })
        }
    }
}
impl CustomDelegate {
    fn make_manuever_card(game_state: &mut GameState, data: &FilledCardResponse, card: &Arc<CardInstance>, user: &(Teams, i32)) {
        let cost = card.get_cost(game_state, user.1);
        // take for cost
        game_state.edit::<StateEnergy>(|x| {
            // get the energy state
            let Some(energy_cur_max) = x.all_players.get_mut(&user.1) else {
                println!("Could not find 'Energy' for UID: {}", user.1);
                return;
            };

            energy_cur_max.0 = energy_cur_max.0 - cost;
        });

        // creates an event runner to all the events on
        let mut event_runner = CardEventRunner::new();

        // get the attributes out of this card
        let atts_mods = card.get_attributes_modifiers(&game_state, user.1);
        let atts_evnt = card.get_attributes_events(&game_state, user.1);

        // iterate over each mod and add it and its data to the runner
        for i in 0..atts_mods.len() {
            event_runner.enqueue_modifier(&atts_mods[i], &data.modifiers[i]);
        }

        // iterate over each event and add it and its data to the runner
        for i in 0..atts_evnt.len() {
            event_runner.enqueue_event(&atts_evnt[i], &data.event[i]);
        }

        // enqueue the clear flag for
        event_runner.enqueue_clear_modifiers(&ModifierClearFlag::Play);

        // run all inside runner
        event_runner.post_and_drain(game_state);
    }
    fn make_manuever_move(game_state: &mut GameState, delta: &Directions, user: &(Teams, i32)) {
        // pull out the constants
        const MOVE_COST: i32 = 1;

        // change the position
        game_state.edit::<StatePositionEntities>(|x| {
            // get the position state
            let Some(position) = x.positions.get(&user.1) else {
                println!("Could not find 'Position' for UID: {}", user.1);
                return;
            };

            // insert the new position into the data
            x.positions
                .insert(user.1, GameBoard::do_move(&user.0, position, delta));
        });

        // remove the energy needed to move
        game_state.edit::<StateEnergy>(|x| {
            // get the energy state
            let Some(energy_cur_max) = x.all_players.get_mut(&user.1) else {
                println!("Could not find 'Energy' for UID: {}", user.1);
                return;
            };

            energy_cur_max.0 = energy_cur_max.0 - MOVE_COST;
        });
    }
    fn make_manuever_end(game_state: &mut GameState) {
        // set the current terminated state to true
        game_state.edit::<StateTerminated>(|x| {
            x.is_terminated = true;
        })
    }
}
