use core::{
    collections::{event_queue::EventQueue, game_state::GameState, vector2_int::Vector2Int},
    dumpster_engine::NetworkModes,
    gameplay::{
        ecs::traits::ecs_event_reciever::{self, InstanceLimiter},
        world_context::WorldContext,
    },
    random::Random,
};

use built_in_state::state_network::StateNetwork;
use ecs_event::global_ecs_system_event_reciever;
use hecs::World;
use serde::{Deserialize, Serialize};

use crate::{
    cards::deck_library::DeckLibrary,
    game_events::GameEvents,
    state::{
        host::{state_enounter_mode::StateEncounter, state_shop::StateShop},
        state_controller::StateController,
        state_deck::{Deck, StateDeck},
        state_energy::StateEnergy,
        state_position_player::StatePositionEntities,
        state_score::StateScore,
        state_teams::{StateTeamAssignments, Teams},
    },
};

#[derive(Default)]
#[global_ecs_system_event_reciever(GameEvents)]
pub struct Listener {}

// Impl - Instance
impl InstanceLimiter for Listener {
    fn is_enabled(&mut self, _game_state: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _game_state: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all_host()
    }
}
// Impl - Listener
impl ecs_event_reciever::EventReciever<GameEvents> for Listener {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut WorldContext, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::InitializeShop(shop) => {
                // log
                println!("Shop Initialized");

                //
                game_state.edit::<StateShop>(|x| {
                    // set the active shop
                    x.shop = shop.clone()
                });
            }
            _ => {}
        }
    }
}
