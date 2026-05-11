use crate::game_events::GameEvents;
use crate::state::host::state_currency::StateCurrency;
use curio_core::{
    collections::{event_queue::Nerve, ledger::Ledger},
    network_modes::NetworkModes,
};
use gameplay::context_3d::Context3D;
use gameplay::traits::{impulse::Impulse, scope::Scope};
use impulse::impulse;

#[derive(Default)]
#[impulse(GameEvents)]
pub struct ECsystemGamePointScored {}

impl Scope for ECsystemGamePointScored {
    fn is_enabled(&mut self, _: &mut Ledger) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut Ledger) -> Vec<NetworkModes> {
        NetworkModes::all_host()
    }
}
impl Impulse<GameEvents> for ECsystemGamePointScored {
    fn dequeue_event(&mut self, ledger: &mut Ledger, _: &mut Context3D, event_queue: &mut Nerve, event: &GameEvents) {
        match event {
            GameEvents::EncounterPassed => {
                // log
                println!("Encounter Passed");

                // claim rewards
                ledger.write::<StateCurrency>(|x| {
                    x.currency += 100;
                });

                // request leave room
                event_queue.enqueue_event(GameEvents::DidEncounterPass);
            }
            _ => {}
        }
    }
}
