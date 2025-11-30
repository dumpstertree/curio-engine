use core::collections::game_state::GameState;
use mcts::{self, GameState as MCTSGameState, transposition_table::TranspositionHash};
use std::sync::Arc;

use crate::ai::dependencies::{simulation_data_source::SimulationDataSource, simulation_delegate::SimulationDelegate, simulation_hasher::SimulationHasher};

#[derive(Clone)]
pub struct Simulation<T, U>
where
    T: Clone + Sync + Send + 'static,
    U: Clone + Sync + Send + 'static,
{
    // dependencies
    hasher: Arc<Box<dyn SimulationHasher>>,
    delegate: Arc<Box<dyn SimulationDelegate<T, U>>>,
    data_source: Arc<Box<dyn SimulationDataSource<T, U>>>,

    // underlying gamestate
    game_state: GameState,
}
// Impl Fn - Public
impl<T, U> Simulation<T, U>
where
    T: Clone + Sync + Send + 'static,
    U: Clone + Sync + Send + 'static,
{
    pub fn new(delegate: Arc<Box<dyn SimulationDelegate<T, U>>>, data_source: Arc<Box<dyn SimulationDataSource<T, U>>>, hasher: Arc<Box<dyn SimulationHasher>>, game_state: GameState) -> Simulation<T, U> {
        // create the simulation
        Simulation {
            game_state: game_state,
            data_source: data_source,
            delegate: delegate,
            hasher: hasher,
        }
    }
    pub fn get_user(&self) -> U {
        self.data_source.get_cur_user(&self.game_state)
    }
    pub fn get_game_state(&self) -> GameState {
        self.game_state.clone()
    }
}
// Impl - Sync
unsafe impl<T, U> Sync for Simulation<T, U>
where
    T: Clone + Sync + Send + 'static,
    U: Clone + Sync + Send + 'static,
{
}
// Impl - MCTSGameState
impl<T, U> MCTSGameState for Simulation<T, U>
where
    T: Sync + Send + Clone + 'static,
    U: Sync + Send + Clone + 'static,
{
    type Move = T;
    type Player = U;
    type MoveList = Vec<T>;

    fn current_player(&self) -> U {
        self.data_source.get_cur_user(&self.game_state)
    }

    fn available_moves(&self) -> Vec<T> {
        let user = self.data_source.get_cur_user(&self.game_state);
        return self
            .data_source
            .get_all_simulation_actions(&self.game_state, &user);
    }

    fn make_move(&mut self, mov: &T) {
        let user = self.data_source.get_cur_user(&self.game_state);
        self.delegate.simulate(&mut self.game_state, &user, mov);
    }
}
// Impl TranspositionHash
impl<T, U> TranspositionHash for Simulation<T, U>
where
    T: Clone + Sync + Send + 'static,
    U: Clone + Sync + Send + 'static,
{
    fn hash(&self) -> u64 {
        // get the hash from the custom hasher
        self.hasher.hash(&self.game_state)
    }
}
