use core::{collections::game_state::GameState, dumpster_engine::NetworkModes};

pub trait Scope {
    fn is_enabled(&mut self, game_state: &mut GameState) -> bool;
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<NetworkModes>;
}
