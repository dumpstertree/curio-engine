use curio_core::collections::game_state::Ledger;

pub trait SimulationDataSource<T, U>
where
    T: Clone + Sync + Send + 'static,
    U: Clone + Sync + Send + 'static,
{
    fn get_cur_user(&self, game_state: &Ledger) -> U;
    fn get_all_simulation_actions(&self, game_state: &Ledger, user: &U) -> Vec<T>;
}
