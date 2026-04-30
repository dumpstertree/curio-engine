use gameplay::{
    context_3d::Context3D,
    traits::{habit::Habit, scope::Scope},
};
use habit::habit;

use curio_core::{
    built_in::record::{sys_record_camera::SysRecordCamera, sys_record_network::SysRecordNetwork},
    collections::{event_queue::EventQueue, ledger::Ledger},
    network_modes::NetworkModes,
};
use std::vec;

use crate::{
    cards::deck_library::DeckLibrary,
    exploration::exploration_path::Exploration,
    game_events::GameEvents,
    state::host::{state_currency::StateCurrency, state_deck_exploration::StateDeckExploration, state_health_exploration::StateHealthExploration},
};

#[habit]
pub struct Instance {}
impl Scope for Instance {
    fn is_enabled(&mut self, _ledger: &mut Ledger) -> bool {
        true
    }
    fn run_on_instance(&mut self, _ledger: &mut Ledger) -> Vec<NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
impl Habit for Instance {
    fn enable(&mut self, ledger: &mut Ledger, _: &mut Context3D, event_queue: &mut EventQueue) {
        println!("Instance: {}. Host Startup", ledger.instance_id);

        // set resolution
        ledger.write::<SysRecordCamera>(|x| {
            x.resolution_width = 1920 / 1;
            x.resolution_height = 1080 / 1;
        });

        ledger.write::<StateCurrency>(|x| {
            x.currency = 100;
        });

        // get state
        let state_network = ledger.read::<SysRecordNetwork>();

        // add starting decks for each player - eventually load this from disc
        ledger.write::<StateDeckExploration>(|x| {
            for id in state_network.peer_instance_ids() {
                x.deck.insert(*id, DeckLibrary::get_player_deck_standard());
            }
        });
        ledger.write::<StateHealthExploration>(|x| {
            for id in state_network.peer_instance_ids() {
                x.all.insert(*id, (7, 7));
            }
        });

        // open exploration
        event_queue.enqueue_event(GameEvents::InitializeExploration(Exploration::random()));
    }
}
