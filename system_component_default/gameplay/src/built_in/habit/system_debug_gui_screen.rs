use curio_core::{Ledger, Nerve, NetworkModes, built_in::record::sys_record_debug::SysRecordDebug};

use crate::{
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
        ledger.read::<SysRecordDebug>().is_inspecting
    }
    fn run_on_instance(&mut self, _ledger: &mut Ledger) -> Vec<NetworkModes> {
        NetworkModes::all()
    }
}
impl Habit for Instance {
    fn tick(&mut self, _: &mut Ledger, _: &mut Context3D, _: &mut Nerve) {
        // // get gpu data
        // let sys_config = SystemGPU::get_config();
        // let sys_window = SystemGPU::get_window();

        // // edit state
        // ledger.write::<SysRecordDebugGui>(|x| {
        //     x.append(format!("Resolution: ({}, {})", sys_config.width, sys_config.height));
        //     x.append(format!("Screen Size: ({}, {})", sys_window.inner_size().width, sys_window.inner_size().height));
        // });
    }
}
