use curio_core::collections::{event_queue::EventQueue, game_state::Ledger};

use crate::{context_3d::Context3D, traits::scope::Scope};

pub trait Impulse<T>: Scope
where
    T: Clone,
{
    fn dequeue_event(&mut self, game_state: &mut Ledger, world: &mut Context3D, event_queue: &mut EventQueue, event: &T);
}
