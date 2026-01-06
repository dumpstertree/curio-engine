use core::collections::{event_queue::EventQueue, game_state::GameState};

use crate::{traits::instance_scope::InstanceLimiter, world_context::WorldContext};

pub trait EventReciever<T>: InstanceLimiter
where
    T: Clone,
{
    fn dequeue_event(&mut self, game_state: &mut GameState, world: &mut WorldContext, event_queue: &mut EventQueue, event: &T);
}
