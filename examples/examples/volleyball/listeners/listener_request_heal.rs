use crate::game_events::GameEvents;
use crate::listeners::listener_ui_set_mode::UITypes;
use crate::state::host::state_currency::StateCurrency;
use crate::state::host::state_heat::StateHeat;
use crate::state::peer::state_peer_entity_ids::{EntityIDTypes, StateEntityIDs};
use crate::state::state_score::StateScore;
use crate::state::state_teams::StateTeamAssignments;
use core::gameplay::ecs::traits::ecs_event_reciever::{self, InstanceLimiter};
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_event::global_ecs_system_event_reciever;
use hecs::World;

#[derive(Default)]
#[global_ecs_system_event_reciever(GameEvents)]
pub struct Listener {}

impl InstanceLimiter for Listener {
    fn is_enabled(&mut self, _: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        NetworkModes::all_host()
    }
}
impl ecs_event_reciever::EventReciever<GameEvents> for Listener {
    fn dequeue_event(&mut self, game_state: &mut GameState, world: &mut World, event_queue: &mut EventQueue, event: &GameEvents) {
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
