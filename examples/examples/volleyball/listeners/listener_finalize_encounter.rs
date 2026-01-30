use curio_core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};

use ecs_event::impulse;
use gameplay::{
    context_3d::Context3D,
    traits::{impulse::Impulse, scope::Scope},
};

use crate::{
    game_events::GameEvents,
    state::{
        host::{state_deck_exploration::StateDeckExploration, state_health_exploration::StateHealthExploration, state_heat::StateHeat},
        state_controller::StateController,
        state_deck::StateDeck,
        state_energy::StateEnergy,
        state_position_player::StatePositionEntities,
        state_score::StateScore,
        state_teams::StateTeamAssignments,
    },
};

#[derive(Default)]
#[impulse(GameEvents)]
pub struct Listener {}

// Impl - Instance
impl Scope for Listener {
    fn is_enabled(&mut self, _game_state: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _game_state: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all_host()
    }
}
// Impl - Listener
impl Impulse<GameEvents> for Listener {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut Context3D, _: &mut EventQueue, event: &GameEvents) {
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

                let state_teams = game_state.get::<StateTeamAssignments>();
                let state_score = game_state.get::<StateScore>();

                // edit health based on encounter change
                game_state.edit::<StateHealthExploration>(|x| {
                    // iterate over each
                    for team_user_id in &state_teams.team_assignments {
                        // score for encounter
                        let Some(score_encounter) = state_score.all_scores.get(&team_user_id.0) else {
                            continue;
                        };

                        // iterate over each user in id
                        for user_id in team_user_id.1 {
                            // get the score for the exploration to edit
                            let Some(score_exploration) = x.all.get_mut(&user_id) else {
                                continue;
                            };

                            // set the exploration score to the encounter score
                            score_exploration.0 = *score_encounter;
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
                // edit scores
                game_state.edit::<StateScore>(|x| x.all_scores.clear());
                // edit scores
                game_state.edit::<StateHeat>(|x| x.all_players.clear());
            }
            _ => {}
        }
    }
}
