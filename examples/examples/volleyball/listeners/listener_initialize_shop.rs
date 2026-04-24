use curio_core::{
    collections::network_modes::NetworkModes,
    collections::{event_queue::EventQueue, ledger::Ledger},
};

use gameplay::{
    context_3d::Context3D,
    traits::{impulse::Impulse, scope::Scope},
};
use impulse::impulse;

use crate::{game_events::GameEvents, state::host::state_shop::StateShop};

#[derive(Default)]
#[impulse(GameEvents)]
pub struct Listener {}

// Impl - Instance
impl Scope for Listener {
    fn is_enabled(&mut self, _ledger: &mut Ledger) -> bool {
        true
    }
    fn run_on_instance(&mut self, _ledger: &mut Ledger) -> Vec<NetworkModes> {
        NetworkModes::all_host()
    }
}
// Impl - Listener
impl Impulse<GameEvents> for Listener {
    fn dequeue_event(&mut self, ledger: &mut Ledger, _: &mut Context3D, _event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::InitializeShop(shop) => {
                // log
                println!("Shop Initialized");

                //
                ledger.edit::<StateShop>(|x| {
                    // set the active shop
                    x.shop = shop.clone()
                });
            }
            _ => {}
        }
    }
}
