use hecs::World;

use crate::collections::{event_queue::EventQueue, game_state::GameState};

pub trait EventReciever<T>
where
    T: Clone,
{
    fn dequeue_event(&mut self, game_state: &mut GameState, world: &mut World, event_queue: &mut EventQueue, event: &T);
}
