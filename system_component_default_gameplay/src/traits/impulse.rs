use curio_core::collections::{event_queue::EventQueue, game_state::GameState};

use crate::{traits::scope::Scope, context_3d::Context3D};

pub trait Impulse<T>: Scope
where
    T: Clone,
{
    fn dequeue_event(&mut self, game_state: &mut GameState, world: &mut Context3D, event_queue: &mut EventQueue, event: &T);
}
