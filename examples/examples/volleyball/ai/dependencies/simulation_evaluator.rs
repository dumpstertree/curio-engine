use curio_core::collections::ledger::Ledger;

pub trait SimulationEvaluator<T, U>
where
    T: Clone + Sync + Send + 'static,
    T: Clone + Sync + Send + 'static,
{
    fn evaluate(&self, ledger: &Ledger, user: U, previous_moves: &Vec<T>) -> i64;
}
