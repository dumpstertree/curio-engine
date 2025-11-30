use core::collections::game_state::GameState;

pub trait SimulationDelegate<T, U>
where
    T: Clone + Sync + Send + 'static,
    U: Clone + Sync + Send + 'static,
{
    fn simulate(&self, game_state: &mut GameState, user: &U, manuever: &T);
}
