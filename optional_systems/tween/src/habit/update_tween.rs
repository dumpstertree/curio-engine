use crate::{
    built_in::{
        tween_transform2d_position::TweenTransform2DPosition, tween_transform2d_rotation::TweenTransform2DRotation, tween_transform2d_scale::TweenTransform2DScale, tween_transform3d_position::TweenTransform3DPosition, tween_transform3d_rotation::TweenTransform3DRotation,
        tween_transform3d_scale::TweenTransform3DScale,
    },
    facet::tween::Tween,
};
use curio_core::{Ledger, Nerve, NetworkModes};
use gameplay::{
    built_in::facet::transform::{transform2d::Transform2D, transform3d::Transform3D},
    context_3d::Context3D,
    traits::{facet_common::FacetCommon, habit::Habit, scope::Scope},
    traits_internal::world_context_common::ContextCommon,
};
use habit::habit;

#[habit]
pub struct Instance;
impl Scope for Instance {
    fn is_enabled(&mut self, _ledger: &mut Ledger) -> bool {
        true
    }

    fn run_on_instance(&mut self, _ledger: &mut Ledger) -> Vec<NetworkModes> {
        NetworkModes::all()
    }
}
impl Habit for Instance {
    fn tick(&mut self, ledger: &mut Ledger, context: &mut Context3D, _: &mut Nerve) {
        // get the deltatime to progress by
        let dt = ledger.time().scaled_delta_time;

        // update all tween data
        context.edit::<&mut Tween>(|q| {
            for (_, t) in q {
                // not enabled
                if !t.form().enabled_in_hierachy() {
                    continue;
                }
                // update
                t.update(dt);
            }
        });

        // pull out tween data and assign to target of type transform2d
        context.edit::<(&Tween, &mut Transform2D)>(|q| {
            for (_, (tween, transform)) in q {
                // not enabled
                if !transform.form().enabled_in_hierachy() {
                    continue;
                }
                //update tween
                for t in &tween.tweens {
                    if let Some(p) = t.as_any().downcast_ref::<TweenTransform2DPosition>() {
                        transform.position = p.p_t;
                    } else if let Some(r) = t.as_any().downcast_ref::<TweenTransform2DRotation>() {
                        transform.rotation = r.p_t;
                    } else if let Some(s) = t.as_any().downcast_ref::<TweenTransform2DScale>() {
                        transform.scale = s.p_t;
                    }
                }
            }
        });

        // pull out tween data and assign to target of type transform3d
        context.edit::<(&Tween, &mut Transform3D)>(|q| {
            for (_, (tween, transform)) in q {
                // not enabled
                if !transform.form().enabled_in_hierachy() {
                    continue;
                }
                // update tween
                for t in &tween.tweens {
                    if let Some(p) = t.as_any().downcast_ref::<TweenTransform3DPosition>() {
                        transform.position = p.p_t;
                    } else if let Some(r) = t.as_any().downcast_ref::<TweenTransform3DRotation>() {
                        transform.rotation = r.p_t;
                    } else if let Some(s) = t.as_any().downcast_ref::<TweenTransform3DScale>() {
                        transform.scale = s.p_t;
                    }
                }
            }
        });
    }
}
