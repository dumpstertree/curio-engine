use crate::UIViewTypes;
use crate::game_events::GameEvents;
use crate::state::host::state_exploration::StateExploration;
use curio_core::{
    collections::network_modes::NetworkModes,
    collections::{event_queue::EventQueue, ledger::Ledger},
};
use gameplay::context_3d::Context3D;
use gameplay::{
    built_in::impulse::ui_events::UIEvents,
    traits::{impulse::Impulse, scope::Scope},
};
use impulse::impulse;

#[derive(Default)]
#[impulse(GameEvents)]
pub struct Listener {}

impl Scope for Listener {
    fn is_enabled(&mut self, _: &mut Ledger) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut Ledger) -> Vec<NetworkModes> {
        NetworkModes::all_peer()
    }
}
impl Impulse<GameEvents> for Listener {
    fn dequeue_event(&mut self, ledger: &mut Ledger, _world: &mut Context3D, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::ExplorationDidPickRoomStart => {
                // turn off selection value
                ledger.write::<StateExploration>(|x| {
                    x.is_selecting_next = true;
                });

                event_queue.enqueue_event(UIEvents::Open(UIViewTypes::PanelExploration));
            }
            _ => {}
        }
    }
}
