use curio_core::{Ledger, Nerve, NetworkModes, built_in::record::sys_record_debug::SysRecordDebug};
use gameplay::{
    built_in::facet::transform::transform3d::Transform3D,
    context_3d::Context3D,
    traits::{habit::Habit, scope::Scope},
    traits_internal::world_context_common::ContextCommon,
};
use habit::habit;

use crate::{DrawCallLight, Light, SysRecordLights};

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
        // iterate over each camera in context
        world.edit::<(&Transform3D, &Light)>(|q| {
            // update records to match cameras in context
            for (_entity, (transform, light)) in q {
                state.write::<SysRecordLights>(|x| {
                    let mut l = DrawCallLight::default();
                    l.color = [light.color.as_r_01(), light.color.as_g_01(), light.color.as_b_01()];
                    l.direction = [light.direction.x, light.direction.y, light.direction.z];
                    l.intensity = light.intensity;
                    l.light_type = light.asset;
                    l.radius = light.radius;
                    l.position = [transform.position.x, transform.position.y, transform.position.z];

                    x.all_lights.push(l);
                });
            }
        });
    }
}
