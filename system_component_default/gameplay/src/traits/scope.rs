use curio_core::{Ledger, NetworkModes};

pub trait Scope {
    fn is_enabled(&mut self, ledger: &mut Ledger) -> bool;
    fn run_on_instance(&mut self, ledger: &mut Ledger) -> Vec<NetworkModes>;
}
