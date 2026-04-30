use crate::{
    game_events::GameEvents,
    state::{state_deck::StateDeck, state_energy::StateEnergy},
};
use curio_core::{
    collections::{event_queue::EventQueue, ledger::Ledger},
    network_modes::NetworkModes,
};
use gameplay::{
    context_3d::Context3D,
    traits::{impulse::Impulse, scope::Scope},
};
use impulse::impulse;
use serde::de;

#[derive(Default)]
#[impulse(GameEvents)]
pub struct ImpulseInstance {}
impl Scope for ImpulseInstance {
    fn is_enabled(&mut self, _: &mut Ledger) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut Ledger) -> Vec<NetworkModes> {
        NetworkModes::all_host()
    }
}
impl Impulse<GameEvents> for ImpulseInstance {
    fn dequeue_event(&mut self, ledger: &mut Ledger, _: &mut Context3D, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::RequestBurnCard(user_guid, card_guid) => {
                ledger.write::<StateDeck>(|x| {
                    if let Some(deck) = x.deck.get_mut(user_guid) {
                        let card_instance = deck.get_instance(*card_guid);
                        if !card_instance.get_burnable() {
                            return;
                        }
                        deck.burn_card(&card_instance);
                    }
                });

                ledger.write::<StateEnergy>(|x| {
                    if let Some(energy) = x.all_players.get_mut(user_guid) {
                        energy.0 += 1;
                    }
                });
            }
            _ => {}
        }
    }
}
