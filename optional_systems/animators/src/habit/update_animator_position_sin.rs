use crate::facet::animator::animator_position_sin::AnimatorPositionSin;
use curio_core::{Ledger, Nerve, NetworkModes};
use gameplay::{
    built_in::facet::transform::transform2d::Transform2D,
    context_3d::Context3D,
    traits::{facet_common::FacetCommon, habit::Habit, scope::Scope},
    traits_internal::world_context_common::ContextCommon,
};
use habit::habit;
use time::ExtensionsLedger;

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
    fn tick(&mut self, ledger: &mut Ledger, context3d: &mut Context3D, _: &mut Nerve) {
        // get state
        let time = ledger.time().scaled_time as f32;

        // edit the forms in context
        context3d.edit::<(&AnimatorPositionSin, &mut Transform2D)>(|x| {
            for (_, (animator, transform2d)) in x {
                // guard - form disabled in hierarchy
                if !animator.form().enabled_in_hierachy() {
                    continue;
                }

                // if enabled update else hold the midpoint of the range
                let sin = if animator.enabled {
                    f32::sin(animator.speed * time)
                } else {
                    0.0
                };
                let delta = animator.max - animator.min;
                let target = animator.min + (delta * ((sin + 1.0) / 2.0));
                transform2d.position = target.to_vector2();
            }
        });
    }
}
