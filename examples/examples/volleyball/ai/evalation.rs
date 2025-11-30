use std::{marker::PhantomData, sync::Arc};

use mcts::SearchHandle;

use crate::ai::{dependencies::simulation_evaluator::SimulationEvaluator, mcts::MCTS, simulation::Simulation};

/// A dependecy for MCTS.
/// Used to evaluate the score for a given simulation.
pub struct Evaluator<T, U>
where
    T: Clone + Sync + Send + 'static,
    U: Clone + Sync + Send + 'static,
{
    // phantom data for types
    phantom_t: PhantomData<T>,
    phantom_u: PhantomData<U>,

    // the custom evaluator we are going to use
    custom_evaluator_instance: Arc<Box<dyn SimulationEvaluator<T, U>>>,
}
// Imple Fns - Public
impl<T, U> Evaluator<T, U>
where
    T: Clone + Sync + Send + 'static,
    U: Clone + Sync + Send + 'static,
{
    pub fn new(custom_evaluator_instance: Arc<Box<dyn SimulationEvaluator<T, U>>>) -> Evaluator<T, U> {
        // create a new instance
        Evaluator {
            // basic phantoms
            phantom_t: PhantomData::default(),
            phantom_u: PhantomData::default(),

            // pass in the evaluator instance
            custom_evaluator_instance,
        }
    }
}
// Imple Fns - Evaluator<MyMCTS<T, U>>
impl<T, U> mcts::Evaluator<MCTS<T, U>> for Evaluator<T, U>
where
    T: Clone + Sync + Send + 'static,
    U: Clone + Sync + Send + 'static,
{
    type StateEvaluation = i64;

    fn evaluate_new_state(&self, simulation: &Simulation<T, U>, moves: &Vec<T>, _: Option<SearchHandle<MCTS<T, U>>>) -> (Vec<()>, i64) {
        (
            // evaluate from the custom evaluator and return it with the num of moves for mcts to do its thing
            vec![(); moves.len()],
            self.custom_evaluator_instance
                .evaluate(&simulation.get_game_state(), simulation.get_user(), moves),
        )
    }

    fn interpret_evaluation_for_player(&self, evaln: &i64, _player: &U) -> i64 {
        *evaln
    }

    fn evaluate_existing_state(&self, _state: &Simulation<T, U>, evaln: &i64, _handle: SearchHandle<MCTS<T, U>>) -> i64 {
        *evaln
    }
}
// Impl Fns - Sync
unsafe impl<T, U> Sync for Evaluator<T, U>
where
    T: Clone + Sync + Send + 'static,
    U: Clone + Sync + Send + 'static,
{
}
