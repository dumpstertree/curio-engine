use core::collections::{event_queue::EventQueue, game_state::GameState};

use crate::{traits::scope::Scope, world_context::WorldContext};

pub trait Impulse<T>: Scope
where
    T: Clone,
{
    fn dequeue_event(&mut self, game_state: &mut GameState, world: &mut WorldContext, event_queue: &mut EventQueue, event: &T);
}
