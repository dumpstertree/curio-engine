use core::collections::game_state::GameState;

pub trait SimulationHasher {
    fn hash(&self, instance: &GameState) -> u64;
}
