use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};

use ecs_event::impulse;
use system_component_default_gameplay::{
    traits::{impulse::Impulse, scope::Scope},
    world_context_3d::WorldContext,
};

use crate::{
    game_events::GameEvents,
    state::host::state_shop::{Shop, StateShop},
};

#[derive(Default)]
#[impulse(GameEvents)]
pub struct Listener {}

// Impl - Instance
impl Scope for Listener {
    fn is_enabled(&mut self, _game_state: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _game_state: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all_host()
    }
}
// Impl - Listener
impl Impulse<GameEvents> for Listener {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut WorldContext, _: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::FinalizeShop(_) => {
                // clear shop
                game_state.edit::<StateShop>(|x| {
                    x.shop = Shop::new(vec![]);
                });
            }
            _ => {}
        }
    }
}
