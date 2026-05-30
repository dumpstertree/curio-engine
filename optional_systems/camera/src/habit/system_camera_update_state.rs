use crate::{Camera, SysRecordCamera};
use curio_core::{Ledger, Nerve, NetworkModes, built_in::record::sys_record_debug::SysRecordDebug};
use gameplay::{
    built_in::facet::transform::transform3d::Transform3D,
    context_3d::Context3D,
    traits::{habit::Habit, scope::Scope},
    traits_internal::world_context_common::ContextCommon,
};
use habit::habit;

#[habit]
pub struct Instance {}
impl Scope for Instance {
    fn is_enabled(&mut self, _ledger: &mut Ledger) -> bool {
        true
    }
    fn run_on_instance(&mut self, _ledger: &mut Ledger) -> Vec<NetworkModes> {
        NetworkModes::all()
    }
}
impl Habit for Instance {
    fn tick(&mut self, state: &mut Ledger, world: &mut Context3D, _: &mut Nerve) {
        // currently using debug controls
        if state.read::<SysRecordDebug>().is_paused {
            return;
        }

        // iterate over each camera in context
        world.edit::<(&mut Transform3D, &Camera)>(|q| {
            // update records to match cameras in context
            for (_entity, (transform, _camera)) in q {
                state.write::<SysRecordCamera>(|x| {
                    x.cameras.position = transform.position;
                    x.cameras.rotation = transform.rotation;
                });
            }
        });
    }
}
