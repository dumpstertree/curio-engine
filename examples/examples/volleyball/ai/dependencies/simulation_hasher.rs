use curio_core::collections::game_state::Ledger;

pub trait SimulationHasher {
    fn hash(&self, instance: &Ledger) -> u64;
}
