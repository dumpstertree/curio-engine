use curio_core::{collections::ledger::Ledger, network_modes::NetworkModes};

pub trait Scope {
    fn is_enabled(&mut self, ledger: &mut Ledger) -> bool;
    fn run_on_instance(&mut self, ledger: &mut Ledger) -> Vec<NetworkModes>;
}
