use curio_core::Nerve;
use curio_core::Ledger;
use hecs::World;

use crate::{context_3d::Context3D, traits::scope::Scope};

pub trait Habit: Scope {
    // data
    fn order(&self, _: &Ledger, _: &World) -> i32 {
        0
    }
    // init
    fn init(&mut self, _: &mut Ledger, _: &mut Context3D, _: &mut Nerve) {}

    // events
    fn debug(&mut self, _: &mut Ledger, _: &mut Context3D, _: &mut Nerve) {}

    // life
    fn enable(&mut self, _: &mut Ledger, _: &mut Context3D, _: &mut Nerve) {}
    fn disable(&mut self, _: &mut Ledger, _: &mut Context3D, _: &mut Nerve) {}

    // tick
    fn will_tick(&mut self, _: &mut Ledger, _: &mut Context3D, _: &mut Nerve) {}
    fn tick(&mut self, _: &mut Ledger, _: &mut Context3D, _: &mut Nerve) {}
    fn did_tick(&mut self, _: &mut Ledger, _: &mut Context3D, _: &mut Nerve) {}
}
