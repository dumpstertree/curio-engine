use std::marker::PhantomData;

use mcts::{CycleBehaviour, transposition_table::ApproxTable, tree_policy::UCTPolicy};

use crate::ai::{evalation::Evaluator, simulation::Simulation};

pub struct MCTS<T, U>
where
    T: Clone + Sync + Send + 'static,
    U: Clone + Sync + Send + 'static,
{
    phantom_t: PhantomData<T>,
    phantom_u: PhantomData<U>,
}
impl<T, U> MCTS<T, U>
where
    T: Clone + Sync + Send + 'static,
    U: Clone + Sync + Send + 'static,
{
    pub fn new() -> MCTS<T, U> {
        MCTS {
            phantom_t: PhantomData::default(),
            phantom_u: PhantomData::default(),
        }
    }
}
impl<T, U> mcts::MCTS for MCTS<T, U>
where
    T: Clone + Sync + Send + 'static,
    U: Clone + Sync + Send + 'static,
{
    type State = Simulation<T, U>;
    type Eval = Evaluator<T, U>;
    type NodeData = ();
    type ExtraThreadData = ();
    type TreePolicy = UCTPolicy;
    type TranspositionTable = ApproxTable<Self>;

    fn cycle_behaviour(&self) -> CycleBehaviour<Self> {
        CycleBehaviour::UseCurrentEvalWhenCycleDetected
    }

    fn virtual_loss(&self) -> i64 {
        0
    }

    fn visits_before_expansion(&self) -> u64 {
        1
    }

    fn node_limit(&self) -> usize {
        std::usize::MAX
    }

    fn select_child_after_search<'a>(&self, children: &'a [mcts::MoveInfo<Self>]) -> &'a mcts::MoveInfo<Self> {
        children
            .into_iter()
            .max_by_key(|child| child.visits())
            .unwrap()
    }

    fn max_playout_length(&self) -> usize {
        1_000_000
    }

    fn on_backpropagation(&self, _evaln: &mcts::StateEvaluation<Self>, _handle: mcts::SearchHandle<Self>) {}
}
