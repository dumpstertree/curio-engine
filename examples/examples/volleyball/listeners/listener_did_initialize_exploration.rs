use crate::UIViewTypes;
use crate::game_events::GameEvents;
use curio_core::collections::{event_queue::EventQueue, ledger::Ledger};
use curio_core::network_modes::NetworkModes;
use gameplay::context_3d::Context3D;
use gameplay::{
    built_in::impulse::ui_events::UIEvents,
    traits::{impulse::Impulse, scope::Scope},
};
use impulse::impulse;

#[derive(Default)]
#[impulse(GameEvents)]
pub struct ECsystemGamePointScored {}

impl Scope for ECsystemGamePointScored {
    fn is_enabled(&mut self, _: &mut Ledger) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut Ledger) -> Vec<NetworkModes> {
        NetworkModes::all_peer()
    }
}
impl Impulse<GameEvents> for ECsystemGamePointScored {
    fn dequeue_event(&mut self, _ledger: &mut Ledger, _: &mut Context3D, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::DidInitializeExploration(_) => {
                event_queue.enqueue_event(UIEvents::Open(UIViewTypes::HudStatus));
            }
            _ => {}
        }
    }
}
