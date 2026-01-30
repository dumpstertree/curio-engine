use ecs_system::habit;
use gameplay::{
    context_3d::Context3D,
    traits::{habit::Habit, scope::Scope},
};

use curio_core::{
    built_in::record::{state_camera::CameraState, state_network::StateNetwork},
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
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
    fn is_enabled(&mut self, game_state: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
impl Habit for Instance {
    fn enable(&mut self, game_state: &mut GameState, _: &mut Context3D, event_queue: &mut EventQueue) {
        println!("Instance: {}. Host Startup", game_state.instance_id);

        // set resolution
        game_state.edit::<CameraState>(|x| {
            x.resolution_width = 1920 / 1;
            x.resolution_height = 1080 / 1;
        });

        game_state.edit::<StateCurrency>(|x| {
            x.currency = 100;
        });

        // get state
        let state_network = game_state.get::<StateNetwork>();

        // add starting decks for each player - eventually load this from disc
        game_state.edit::<StateDeckExploration>(|x| {
            for id in state_network.peer_instance_ids() {
                x.deck.insert(*id, DeckLibrary::get_player_deck_standard());
            }
        });
        game_state.edit::<StateHealthExploration>(|x| {
            for id in state_network.peer_instance_ids() {
                x.all.insert(*id, (7, 7));
            }
        });

        // open exploration
        event_queue.enqueue_event(GameEvents::InitializeExploration(Exploration::random()));
    }
}
