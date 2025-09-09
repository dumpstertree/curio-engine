use crate::{game_events::GameEvents, state::state_score::StateScore};
use core::gameplay::ecs::traits::ecs_event_reciever;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};
use ecs_event::global_ecs_system_event_reciever;
use ecs_system::global_ecs_system;
use hecs::World;

#[global_ecs_system]
pub struct ECSSystemGamePointScored {}
impl ECSSystemEventless for ECSSystemGamePointScored {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}

#[global_ecs_system_event_reciever(GameEvents)]
impl ecs_event_reciever::EventReciever<GameEvents> for ECSSystemGamePointScored {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut World, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::PointScored(team) => {
                // update score
                game_state.edit::<StateScore>(|x| {
                    // if there is already a score we use that
                    let mut cur_score = 0;
                    if x.all_scores.contains_key(team) {
                        cur_score = x.all_scores[team];
                    }

                    // incriment by 1 point
                    x.all_scores.insert(team.clone(), cur_score + 1);
                });

                // reset the board with the team scored on as the server
                event_queue.enqueue_event(GameEvents::ResetBoard(team.next_team()));
            }
            _ => {}
        }
    }
}
