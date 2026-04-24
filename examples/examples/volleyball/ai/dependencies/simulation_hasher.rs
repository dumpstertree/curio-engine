use curio_core::collections::ledger::Ledger;

pub trait SimulationHasher {
    fn hash(&self, instance: &Ledger) -> u64;
}
