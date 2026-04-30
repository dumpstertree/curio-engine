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
    fn dequeue_event(&mut self, _ledger: &mut Ledger, _world: &mut Context3D, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::ExplorationDidRoomExitHeal(_) => {
                println!("exit heal room");
                // change ui
                // event_queue.enqueue_event(GameEvents::SetUIMode(UITypes::None));
                event_queue.enqueue_event(UIEvents::Close(UIViewTypes::PanelMedic));
            }
            _ => {}
        }
    }
}
