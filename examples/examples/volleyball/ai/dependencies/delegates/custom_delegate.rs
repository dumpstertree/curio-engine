use core::collections::game_state::GameState;
use std::sync::Arc;

use crate::{
    ai::{StateTerminated::StateTerminated, dependencies::delegate::SimulationDelegate},
    ai_resolver::{CardEventRunner, Directions, Move},
    cards::card_instance::CardInstance,
    game_board::GameBoard,
    game_events::FilledCardResponse,
    state::{
        state_energy::StateEnergy,
        state_position_player::StatePositionPlayer,
        state_teams::{StateTeamAssignments, Teams},
        state_turn::StateTurn,
    },
};
pub struct CustomDelegate {}
impl SimulationDelegate<Move, (Teams, i32)> for CustomDelegate {
    // simulates the current move into the gamestate
    fn simulate(&self, game_state: &mut GameState, user: &(Teams, i32), manuever: &Move) {
        // creates an event runner to all all the events on
        let mut event_runner = CardEventRunner::new();

        // each manuever type is handled differently. call the corresponding fn for each manuever
        match manuever {
            Move::Play(card, data) => Self::make_manuever_card(game_state, &data, &card, &mut event_runner),
            Move::EndTurn => Self::make_manuever_end(game_state),
            Move::Move(vector2_int) => Self::make_manuever_move(game_state, &vector2_int),
            Move::Invalid => {}
        }

        //
        let state_turn = game_state.get::<StateTurn>();
        let uid = state_turn.active_instance_id;
        let energy = game_state
            .get::<StateEnergy>()
            .all_players
            .get(&uid)
            .unwrap()
            .0;

        // if we have 0 or less energy we are exhuasted which is a more specific typer of terminated
        let is_exhuasted = energy <= 0;
        if !is_exhuasted {
            return;
        }

        // edit gamestate for represent our exhausted state
        game_state.edit::<StateTerminated>(|x| {
            x.is_terminated = true;
            x.is_exhuasted = true;
        })
    }
}
impl CustomDelegate {
    fn make_manuever_card(game_state: &mut GameState, data: &FilledCardResponse, card: &Arc<CardInstance>, event_runner: &mut CardEventRunner) {
        let state_turn = game_state.get::<StateTurn>();
        let uid = state_turn.active_instance_id;

        // add modifiers
        for modifier in card.get_attributes_modifiers(&game_state, uid) {
            event_runner.enqueue_modifier(&modifier);
        }
        // add event
        let e = card.get_attributes_events(&game_state, uid);
        for i in 0..e.len() {
            let attribute = &e[i];
            let filled_attribute_deps = &data.event[i];

            event_runner.enqueue_event(attribute, filled_attribute_deps);
        }
        // post all
        event_runner.post_and_drain(game_state);
    }
    fn make_manuever_move(game_state: &mut GameState, delta: &Directions) {
        let uid = game_state.get::<StateTurn>().active_instance_id;
        let state_teams = game_state.get::<StateTeamAssignments>();
        let team = state_teams.team_for(&uid).unwrap();

        // change the position
        game_state.edit::<StatePositionPlayer>(|x| {
            // reset energy
            let pos = GameBoard::do_move(&team, x.positions.get(&uid).unwrap(), delta);
            x.positions.insert(uid, pos);
        });

        // remove the energy needed to move
        game_state.edit::<StateEnergy>(|x| {
            let cur = x.all_players[&uid];
            x.all_players.insert(uid, (cur.0 - 1, cur.1));
        });
    }
    fn make_manuever_end(game_state: &mut GameState) {
        // set the current terminated state to true
        game_state.edit::<StateTerminated>(|x| {
            x.is_terminated = true;
        })
    }
}
