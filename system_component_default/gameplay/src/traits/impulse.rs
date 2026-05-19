use crate::{context_3d::Context3D, traits::scope::Scope};
use curio_core::Nerve;
use curio_core::Ledger;

pub trait Impulse<T>: Scope
where
    T: Clone,
{
    fn dequeue_event(&mut self, ledger: &mut Ledger, world: &mut Context3D, event_queue: &mut Nerve, event: &T);
}
