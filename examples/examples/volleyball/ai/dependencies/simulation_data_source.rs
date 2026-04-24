use curio_core::collections::ledger::Ledger;

pub trait SimulationDataSource<T, U>
where
    T: Clone + Sync + Send + 'static,
    U: Clone + Sync + Send + 'static,
{
    fn get_cur_user(&self, ledger: &Ledger) -> U;
    fn get_all_simulation_actions(&self, ledger: &Ledger, user: &U) -> Vec<T>;
}
