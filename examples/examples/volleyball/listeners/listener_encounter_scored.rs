use crate::state::state_teams::Teams;
use crate::{game_events::GameEvents, state::state_score::StateScore};
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_event::global_ecs_system_event_reciever;
use system_component_default_gameplay::world_context::WorldContext;
use system_component_default_gameplay::{
    UIEvents,
    traits::{impulse::Impulse, scope::Scope},
};

#[derive(Default)]
#[global_ecs_system_event_reciever(GameEvents)]
pub struct ECSSystemGamePointScored {}

impl Scope for ECSSystemGamePointScored {
    fn is_enabled(&mut self, _: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        NetworkModes::all_host()
    }
}
impl Impulse<GameEvents> for ECSSystemGamePointScored {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut WorldContext, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::PointScored(team) => {
                // update score
                game_state.edit::<StateScore>(|x| {
                    // if there is already a score we use that
                    let mut cur_score = 0;
                    if x.all_scores.contains_key(&team.next_team()) {
                        cur_score = x.all_scores[&team.next_team()];
                    }

                    // incriment by 1 point
                    x.all_scores.insert(team.next_team().clone(), cur_score - 1);
                });

                // get the score state
                let state_score = game_state.get::<StateScore>();

                // get the scores
                let score_red = state_score.all_scores.get(&Teams::Red).unwrap_or(&99);
                let score_blue = state_score.all_scores.get(&Teams::Blue).unwrap_or(&99);

                // if either score is above the end threshold its over
                let encounter_ended = score_red <= &0 || score_blue <= &0;
                if encounter_ended {
                    if score_red > score_blue {
                        // if we have a higher score we win
                        event_queue.enqueue_event(GameEvents::EncounterPassed);
                    } else {
                        // if the score is the same or less we lost
                        event_queue.enqueue_event(GameEvents::EncounterFailed);
                    }
                } else {
                    // reset the board with the team scored on as the server
                    event_queue.enqueue_event(GameEvents::ResetBoard(team.next_team()));
                }
            }
            _ => {}
        }
    }
}
