use core::collections::game_state::GameState;
use mcts::{MCTSManager, transposition_table::ApproxTable, tree_policy::UCTPolicy};
use std::sync::Arc;

use crate::ai::{
    dependencies::{data_source::SimulationDataSource, delegate::SimulationDelegate, evaluator::SimulationEvaluator, hasher::SimulationHasher},
    evalation::Evaluator,
    mcts::MCTS,
    simulation::Simulation,
};

pub struct AISimulator<T, U>
where
    T: Default + Clone + Sync + Send + 'static,
    U: Clone + Sync + Send + 'static,
{
    data_source: Arc<Box<dyn SimulationDataSource<T, U>>>,
    delegate: Arc<Box<dyn SimulationDelegate<T, U>>>,
    hasher: Arc<Box<dyn SimulationHasher>>,
    evaluator: Arc<Box<dyn SimulationEvaluator<T, U>>>,
    get_game_state: fn(&GameState) -> GameState,
}
impl<T, U> AISimulator<T, U>
where
    T: Default + Clone + Sync + Send + 'static,
    U: Clone + Sync + Send + 'static,
{
    pub fn new(delgate: Box<dyn SimulationDelegate<T, U>>, data_source: Box<dyn SimulationDataSource<T, U>>, hasher: Box<dyn SimulationHasher>, evaluator: Box<dyn SimulationEvaluator<T, U>>, get_game_state: fn(&GameState) -> GameState) -> AISimulator<T, U> {
        AISimulator {
            evaluator: Arc::new(evaluator),
            data_source: Arc::new(data_source),
            delegate: Arc::new(delgate),
            hasher: Arc::new(hasher),
            get_game_state,
        }
    }
    pub fn simulate(&self, bulky_game_state: &GameState, fidelity: Fidelity, threading: Threading) -> T {
        // we take the bulky game state that is filled with lots of extra info from the entire game and we
        // trim out the fat so there is as little info to clone as we can
        let lean_game_state = (self.get_game_state)(bulky_game_state);

        // do more research into what this does
        let policy = UCTPolicy::new(0.5);

        // do more research into what this does
        let table = ApproxTable::new(1024); // tune size

        // create the instance of a simulation to use as the base for all changes we may make
        let starting_simulation = Simulation::new(self.delegate.clone(), self.data_source.clone(), self.hasher.clone(), lean_game_state);

        // create the objec that will evaluate each simulation
        let evaluator = Evaluator::new(self.evaluator.clone());

        // creates the settings and flow for mcts. This currently does not support custom impls
        let mcts = MCTS::new();

        // create the manager that will run the simulations
        let mut manager = MCTSManager::new(starting_simulation, mcts, evaluator, policy, table);

        // playout all simulations
        match threading {
            Threading::Single => manager.playout_n(fidelity as u64),
            Threading::Multi => manager.playout_n_parallel(fidelity as u32, 4),
        }

        // if we found a best move return it otherwise return the default value
        if let Some(best_move) = manager.best_move() { best_move } else { T::default() }
    }
}

pub enum Threading {
    Single,
    Multi,
}
pub enum Fidelity {
    Low = 500,
    Medium = 1000,
    High = 3000,
    Extreme = 10000,
}
