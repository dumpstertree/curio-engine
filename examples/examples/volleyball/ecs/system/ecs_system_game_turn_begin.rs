use crate::{game_events::GameEvents, state::state_energy::StateEnergy};
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    gameplay::ecs::traits::{ecs_event_reciever, ecs_system::ECSSystemEventless},
};
use ecs_event::global_ecs_system_event_reciever;
use ecs_system::global_ecs_system;
use hecs::World;

#[global_ecs_system]
pub struct ECSSystemGameTurnBegin {}
impl ECSSystemEventless for ECSSystemGameTurnBegin {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
}
#[global_ecs_system_event_reciever(GameEvents)]
impl ecs_event_reciever::EventReciever<GameEvents> for ECSSystemGameTurnBegin {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut World, _: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::TurnEnd(id) => {
                // end this turn
                println!("Begin Turn {}", id);

                // update energy
                game_state.edit::<StateEnergy>(|x| {
                    x.cur_energy = x.max_energy;
                });
            }
            _ => {}
        }
    }
}
