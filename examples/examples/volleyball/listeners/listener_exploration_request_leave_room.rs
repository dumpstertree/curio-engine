use crate::game_events::GameEvents;
use crate::state::host::state_exploration::StateExploration;
use curio_core::{
    collections::{event_queue::Nerve, ledger::Ledger},
    network_modes::NetworkModes,
};
use gameplay::{
    context_3d::Context3D,
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
        NetworkModes::all_host()
    }
}
impl Impulse<GameEvents> for ECsystemGamePointScored {
    fn dequeue_event(&mut self, ledger: &mut Ledger, _: &mut Context3D, event_queue: &mut Nerve, event: &GameEvents) {
        match event {
            GameEvents::RequestLeaveExplorationRoom => {
                // exit current room
                let state_exploration = ledger.read::<StateExploration>();
                event_queue.enqueue_event(GameEvents::ExplorationRoomExit(state_exploration.exploration.get_cur_room()));

                // change state
                let state_exploration = ledger.read::<StateExploration>();
                event_queue.enqueue_event(GameEvents::ExplorationPickRoomStart(state_exploration.exploration.clone()));

                // did pick room
                event_queue.enqueue_event(GameEvents::ExplorationDidPickRoomStart);
            }
            _ => {}
        }
    }
}
