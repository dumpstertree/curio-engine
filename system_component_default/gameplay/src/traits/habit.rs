use curio_core::collections::{event_queue::EventQueue, ledger::Ledger};
use hecs::World;

use crate::{context_3d::Context3D, traits::scope::Scope};

pub trait Habit: Scope {
    // data
    fn order(&self, _: &Ledger, _: &World) -> i32 {
        0
    }
    // init
    fn init(&mut self, _: &mut Ledger, _: &mut Context3D, _: &mut EventQueue) {}

    // events
    fn debug(&mut self, _: &mut Ledger, _: &mut Context3D, _: &mut EventQueue) {}

    // life
    fn enable(&mut self, _: &mut Ledger, _: &mut Context3D, _: &mut EventQueue) {}
    fn disable(&mut self, _: &mut Ledger, _: &mut Context3D, _: &mut EventQueue) {}

    // tick
    fn will_tick(&mut self, _: &mut Ledger, _: &mut Context3D, _: &mut EventQueue) {}
    fn tick(&mut self, _: &mut Ledger, _: &mut Context3D, _: &mut EventQueue) {}
    fn did_tick(&mut self, _: &mut Ledger, _: &mut Context3D, _: &mut EventQueue) {}
}
