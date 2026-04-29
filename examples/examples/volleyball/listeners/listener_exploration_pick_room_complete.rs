use crate::game_events::GameEvents;
use crate::state::host::state_exploration::StateExploration;
use curio_core::{
    collections::network_modes::NetworkModes,
    collections::{event_queue::EventQueue, ledger::Ledger},
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
    fn dequeue_event(&mut self, ledger: &mut Ledger, _: &mut Context3D, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::ExplorationPickRoomComplete(room) => {
                // // edit the encounter state to move to the next room
                ledger.write::<StateExploration>(|x| {
                    // progress the exploration
                    x.exploration.next(&room.guid);
                });

                // enter current room
                let state_exploration = ledger.read::<StateExploration>();
                event_queue.enqueue_event(GameEvents::ExplorationRoomEnter(state_exploration.exploration.get_cur_room()));

                // close the ui
                event_queue.enqueue_event(GameEvents::ExplorationDidPickRoomComplete);
            }
            _ => {}
        }
    }
}
