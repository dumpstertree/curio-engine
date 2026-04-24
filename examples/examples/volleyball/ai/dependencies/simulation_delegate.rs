use curio_core::collections::game_state::Ledger;

pub trait SimulationDelegate<T, U>
where
    T: Clone + Sync + Send + 'static,
    U: Clone + Sync + Send + 'static,
{
    fn simulate(&self, game_state: &mut Ledger, user: &U, manuever: &T);
}
