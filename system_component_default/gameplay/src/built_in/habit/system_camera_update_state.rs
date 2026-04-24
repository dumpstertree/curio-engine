use curio_core::{
    built_in::record::{sys_record_camera::SysRecordCamera, sys_record_debug::SysRecordDebug},
    collections::network_modes::NetworkModes,
    collections::{event_queue::EventQueue, ledger::Ledger},
};
// use habit::habit;

use crate::{
    built_in::facet::{camera::Camera, transform::transform3d::Transform3D},
    context_3d::Context3D,
    traits::{habit::Habit, scope::Scope},
    traits_internal::world_context_common::ContextCommon,
};

// #[global_ecs_system]
#[derive(Default)]
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
    fn tick(&mut self, state: &mut Ledger, world: &mut Context3D, _: &mut EventQueue) {
        if state.get::<SysRecordDebug>().is_paused {
            return;
        }
        world.edit::<(&mut Transform3D, &Camera)>(|q| {
            //
            for (_entity, (transform, _camera)) in q {
                state.edit::<SysRecordCamera>(|x| {
                    x.cameras.position = transform.position;
                    x.cameras.rotation = transform.rotation;
                });
            }
        });
        // for (_, (transform, _camera)) in world.query_mut::<(&mut Transform, &Camera)>() {
        //     state.edit::<CameraState>(|x| {
        //         x.cameras.position = transform.position;
        //         x.cameras.rotation = transform.rotation;
        //     });
        // }
    }
}
