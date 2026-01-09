use crate::game_events::GameEvents;
use crate::state::host::state_currency::StateCurrency;
use crate::state::state_score::StateScore;
use crate::state::state_teams::StateTeamAssignments;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_event::impulse;
use system_component_default_gameplay::traits::{impulse::Impulse, scope::Scope};
use system_component_default_gameplay::world_context_3d::WorldContext;

#[derive(Default)]
#[impulse(GameEvents)]
pub struct Listener {}

impl Scope for Listener {
    fn is_enabled(&mut self, _: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        NetworkModes::all_host()
    }
}
impl Impulse<GameEvents> for Listener {
    fn dequeue_event(&mut self, game_state: &mut GameState, world: &mut WorldContext, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::RequestHeal(user_guid) => {
                let state_currency = game_state.get::<StateCurrency>();
                if state_currency.currency < 100 {
                    println!("Not enough Currency. Require 100 have {}", state_currency.currency);
                    return;
                }

                let state_teams = game_state.get::<StateTeamAssignments>();
                game_state.edit::<StateCurrency>(|x| {
                    x.currency -= 100;
                });
                game_state.edit::<StateScore>(|x| {
                    if let Some(team) = state_teams.team_for(user_guid) {
                        if let Some(score) = x.all_scores.get_mut(&team) {
                            *score += 1;
                        }
                    }
                });
            }
            _ => {}
        }
    }
}
