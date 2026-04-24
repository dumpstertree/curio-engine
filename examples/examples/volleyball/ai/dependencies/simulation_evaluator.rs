use curio_core::collections::game_state::Ledger;

pub trait SimulationEvaluator<T, U>
where
    T: Clone + Sync + Send + 'static,
    T: Clone + Sync + Send + 'static,
{
    fn evaluate(&self, game_state: &Ledger, user: U, previous_moves: &Vec<T>) -> i64;
}
