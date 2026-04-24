use curio_core::collections::{game_state::Ledger, network_modes::NetworkModes};

pub trait Scope {
    fn is_enabled(&mut self, game_state: &mut Ledger) -> bool;
    fn run_on_instance(&mut self, game_state: &mut Ledger) -> Vec<NetworkModes>;
}
