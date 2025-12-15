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
        host::{state_deck_exploration::StateDeckExploration, state_enounter_mode::StateEncounter},
        state_controller::{self, StateController},
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
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut WorldContext, _: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::FinalizeEncounter(_) => {
                // get state
                let state_deck: StateDeck = game_state.get::<StateDeck>();

                // remove any consumed cards
                game_state.edit::<StateDeckExploration>(|x| {
                    // iterate over each deck used in encounter
                    for deck_encounter in &state_deck.deck {
                        // if deck is also in exploration we continue
                        let Some(deck_exploration) = x.deck.get_mut(deck_encounter.0) else {
                            continue;
                        };

                        // for each card in the cosume file of the encounter deck remove it from the exploration deck
                        for card in &deck_encounter.1.pile_consume {
                            deck_exploration.remove_card_from_deck(&card.instance_id);
                        }
                    }
                });

                // do some cleanup removing values we are no longer using

                // clear teams foreach
                game_state.edit::<StateTeamAssignments>(|x| x.team_assignments.clear());
                // clear positions foreach
                game_state.edit::<StatePositionEntities>(|x| x.positions.clear());
                // clear energy foreach
                game_state.edit::<StateEnergy>(|x| x.all_players.clear());
                // clear controller foreach
                game_state.edit::<StateController>(|x| x.all_players.clear());
                // clear decks foreach
                game_state.edit::<StateDeck>(|x| x.deck.clear());
            }
            _ => {}
        }
    }
}
