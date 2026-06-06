use curio_core::{Ledger, Nerve, NetworkModes};

use crate::{
    built_in::facet::transform::{transform2d::update_transform2d, transform3d::update_transform3d},
    context_3d::Context3D,
    traits::{habit::Habit, scope::Scope},
};

#[derive(Default)]
pub struct Instance {}
impl Instance {
    pub fn new() -> Box<Instance> {
        Box::new(Instance {})
    }
}
impl Scope for Instance {
    fn is_enabled(&mut self, ledger: &mut Ledger) -> bool {
        true
    }
    fn run_on_instance(&mut self, _ledger: &mut Ledger) -> Vec<NetworkModes> {
        NetworkModes::all()
    }
}
impl Habit for Instance {
    fn tick(&mut self, ledger: &mut Ledger, world: &mut Context3D, _: &mut Nerve) {
        update_transform2d(world);
        update_transform3d(world);
    }
}
