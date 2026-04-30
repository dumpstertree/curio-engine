use curio_core::{
    ButtonCode,
    built_in::record::{sys_record_debug::SysRecordDebug, sys_record_input::SysRecordInput},
    collections::{event_queue::EventQueue, ledger::Ledger},
    network_modes::NetworkModes,
};

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
    fn is_enabled(&mut self, _ledger: &mut Ledger) -> bool {
        true
    }
    fn run_on_instance(&mut self, _ledger: &mut Ledger) -> Vec<NetworkModes> {
        NetworkModes::all_host()
    }
}
impl Habit for Instance {
    fn tick(&mut self, ledger: &mut Ledger, _: &mut Context3D, _: &mut EventQueue) {
        // get state
        let state_input = ledger.read::<SysRecordInput>();

        // get input button
        let debug_button = state_input.raw.get_button(&ButtonCode::Backquote);
        if debug_button.went_up {
            // flip the toggle
            ledger.write::<SysRecordDebug>(|x| {
                x.is_inspecting = !x.is_inspecting;
            });
        }
    }
}
