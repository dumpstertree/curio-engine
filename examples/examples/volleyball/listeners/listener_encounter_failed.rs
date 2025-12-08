use crate::game_events::GameEvents;
use core::gameplay::ecs::traits::ecs_event_reciever::{self, InstanceLimiter};
use core::gameplay::world_context::WorldContext;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_event::global_ecs_system_event_reciever;
use hecs::World;

#[derive(Default)]
#[global_ecs_system_event_reciever(GameEvents)]
pub struct ECSSystemGamePointScored {}

impl InstanceLimiter for ECSSystemGamePointScored {
    fn is_enabled(&mut self, _: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        NetworkModes::all_host()
    }
}
impl ecs_event_reciever::EventReciever<GameEvents> for ECSSystemGamePointScored {
    fn dequeue_event(&mut self, _game_state: &mut GameState, _: &mut WorldContext, _event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::EncounterFailed => {
                panic!("encounter failed");
            }
            _ => {}
        }
    }
}
