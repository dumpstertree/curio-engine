use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};

// use hecs::World;

use crate::world_context::WorldContext;

// use crate::{
//     collections::{event_queue::EventQueue, game_state::GameState},
//     dumpster_engine::NetworkModes,
//     gameplay::world_context::WorldContext,
// };

pub trait EventReciever<T>: InstanceLimiter
where
    T: Clone,
{
    fn dequeue_event(&mut self, game_state: &mut GameState, world: &mut WorldContext, event_queue: &mut EventQueue, event: &T);
}

pub trait InstanceLimiter {
    fn is_enabled(&mut self, game_state: &mut GameState) -> bool;
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<NetworkModes>;
}
