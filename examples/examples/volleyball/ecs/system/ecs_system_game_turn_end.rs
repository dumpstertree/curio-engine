use ecs_event::global_ecs_system_event_reciever;
use ecs_system::global_ecs_system;
use hecs::World;

use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    gameplay::ecs::traits::{ecs_event_reciever, ecs_system::ECSSystemEventless},
};

use crate::{ game_events::GameEvents};

#[global_ecs_system]
pub struct ECSSystemGameEndTurn {}
impl ECSSystemEventless for ECSSystemGameEndTurn {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
}
#[global_ecs_system_event_reciever(GameEvents)]
impl ecs_event_reciever::EventReciever<GameEvents> for ECSSystemGameEndTurn {
    fn dequeue_event(&mut self, _: &mut GameState, _: &mut World, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::TurnEnd(id) => {
                // end this turn
                println!("End Turn {}", id);

                // todo iterate to the next player

                // begin the next player
                event_queue.enqueue_event(GameEvents::TurnBegin(*id));
            }
            _ => {}
        }
    }
}
