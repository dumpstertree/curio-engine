use built_in_state::{state_camera::CameraState, state_network::StateNetwork};
use ecs_system::global_ecs_system;
use system_component_default_gameplay::{traits::ecs_system::ECSSystemEventless, world_context::WorldContext};

use core::{
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

#[global_ecs_system]
pub struct ECSSystemGameStart {}
impl ECSSystemEventless for ECSSystemGameStart {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut WorldContext) -> bool {
        true
    }
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
    fn enable(&mut self, game_state: &mut GameState, _: &mut WorldContext, event_queue: &mut EventQueue) {
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
