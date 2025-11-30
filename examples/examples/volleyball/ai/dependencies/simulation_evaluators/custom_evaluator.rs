use core::collections::game_state::GameState;

use crate::{
    ai::dependencies::simulation_evaluator::SimulationEvaluator,
    cards::enums::simulation_manuevers::SimulationManuevers,
    game_board::GameBoard,
    state::{other::state_terminated::StateTerminated, state_energy::StateEnergy, state_position_ball::StatePositionBall, state_teams::Teams},
};

pub struct CustomEvaluator {}
impl SimulationEvaluator<SimulationManuevers, (Teams, i32)> for CustomEvaluator {
    fn evaluate(&self, game_state: &GameState, user: (Teams, i32), previous_moves: &Vec<SimulationManuevers>) -> i64 {
        // get states
        let state_position_ball = game_state.get::<StatePositionBall>();
        let state_energy = game_state.get::<StateEnergy>();
        let state_terminated = game_state.get::<StateTerminated>();

        // get bounds
        let min = GameBoard::get_bounds_min(&user.0);
        let max = GameBoard::get_bounds_max(&user.0);

        // check if on my size
        let on_my_side = state_position_ball.row >= min.y && state_position_ball.row <= max.y;

        // if we are exhuasted thats worst case scenerio because it encourages procrastinating
        if state_terminated.is_exhuasted && on_my_side {
            return -999;
        }
        // if we are terminated on our side that means we werent able to return the ball
        if state_terminated.is_terminated && on_my_side {
            return -99;
        }

        // get the cur and max energy for the user
        let Some(energy_cur_max) = state_energy.all_players.get(&user.1) else {
            println!("Unable to find energy data for UID : {}", user.1);
            return 0;
        };

        // calculate all the parts of the scoring fn
        let score_max_energy = energy_cur_max.1 as i64; // more points the more max energy we still have
        let score_moves = -(previous_moves.len() as i64); // less points the more moves it took us to get there

        // create the final the score
        score_max_energy + score_moves
    }
}
