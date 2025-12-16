use core::collections::game_state::GameState;

use crate::{
    ai::dependencies::simulation_evaluator::SimulationEvaluator,
    cards::enums::simulation_manuevers::SimulationManuevers,
    game_board::GameBoard,
    state::{host::state_card_attribute_modifier_stack::StateCardAttributeModifierStack, other::state_terminated::StateTerminated, state_deck::StateDeck, state_energy::StateEnergy, state_position_ball::StatePositionBall, state_teams::Teams},
};

pub struct CustomEvaluator {}
impl SimulationEvaluator<(i32, SimulationManuevers), (Teams, Vec<i32>)> for CustomEvaluator {
    fn evaluate(&self, game_state: &GameState, user: (Teams, Vec<i32>), previous_moves: &Vec<(i32, SimulationManuevers)>) -> i64 {
        // get states
        let state_position_ball = game_state.get::<StatePositionBall>();
        let state_energy = game_state.get::<StateEnergy>();
        let state_terminated = game_state.get::<StateTerminated>();
        let state_modifier_stack = game_state.get::<StateCardAttributeModifierStack>();
        let state_deck = game_state.get::<StateDeck>();

        // get bounds
        let min = GameBoard::get_bounds_min(&user.0);
        let max = GameBoard::get_bounds_max(&user.0);

        // check if on my size
        let on_my_side = state_position_ball.row >= min.y && state_position_ball.row <= max.y;

        // this was causing the enemy to always just end turn
        // if we are exhuasted thats worst case scenerio because it encourages procrastinating
        // if state_terminated.is_exhuasted && on_my_side {
        //     return -999;
        // }
        // if we are terminated on our side that means we werent able to return the ball
        if state_terminated.is_terminated && on_my_side {
            return -99;
        }

        let mut score_max_energy = 0; // more points the more max energy we still have
        let mut score_cur_energy = 0; // more points the more max energy we still have
        let mut score_deck = 0; //less points the more moves it took us to get there
        let mut score_moves = 0;

        for user_uid in user.1 {
            // get the cur and max energy for the user
            let Some(energy_cur_max) = state_energy.all_players.get(&user_uid) else {
                println!("Unable to find energy data for UID : {}", user_uid);
                return 0;
            };

            // get the deck for the user
            let Some(deck) = state_deck.deck.get(&user_uid) else {
                println!("Unable to find deck data for UID : {}", user_uid);
                return 0;
            };

            // get any modifiers for this user
            let mod_stack = state_modifier_stack.get_flat_stack_for_entity(user_uid);

            // println!("energy: {}", (energy_cur_max.1 as i64 + mod_stack.energy as i64));
            // calculate all the parts of the scoring fn
            score_max_energy += (energy_cur_max.1 as i64 + mod_stack.energy as i64) * 2; // more points the more max energy we still have
            score_cur_energy += (energy_cur_max.0 as i64) * 3; // more points the more max energy we still have
            score_deck += deck.hand_consumable.len() as i64; // less points the more moves it took us to get there
            score_moves += -(previous_moves.len() as i64); // less points the more moves it took us to get there
        }

        // put in place incase we ever hit an infite loop
        if score_cur_energy + score_max_energy + score_moves + score_deck > 1000 {
            panic!("unstable score {:?} -> ", previous_moves)
        }
        // create the final the score
        score_max_energy + score_cur_energy + score_deck + score_moves
    }
}
