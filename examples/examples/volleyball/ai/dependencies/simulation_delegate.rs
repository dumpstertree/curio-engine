use curio_core::collections::ledger::Ledger;

pub trait SimulationDelegate<T, U>
where
    T: Clone + Sync + Send + 'static,
    U: Clone + Sync + Send + 'static,
{
    fn simulate(&self, ledger: &mut Ledger, user: &U, manuever: &T);
}
