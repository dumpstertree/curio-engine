use core::collections::game_state::GameState;

pub trait SimulationDataSource<T, U>
where
    T: Clone + Sync + Send + 'static,
    U: Clone + Sync + Send + 'static,
{
    fn get_cur_user(&self, game_state: &GameState) -> U;
    fn get_all_simulation_actions(&self, game_state: &GameState, user: &U) -> Vec<T>;
}
