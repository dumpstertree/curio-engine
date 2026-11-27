use curio_core::{Ledger, Nerve, NetworkModes};
use ext_rendering::{DrawCall, SysRecordRendering};
use gameplay::{
    built_in::facet::transform::transform3d::Transform3D,
    context_3d::Context3D,
    traits::{facet_common::FacetCommon, habit::Habit, scope::Scope},
    traits_internal::world_context_common::ContextCommon,
};
use habit::habit;

use crate::RendererStatic;

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
    fn did_tick(&mut self, state: &mut Ledger, world: &mut Context3D, _: &mut Nerve) {
        // edit the state
        state.write::<SysRecordRendering>(|x| {
            // iterate over each transform3d + renderer
            world.edit::<(&RendererStatic, &Transform3D)>(|query| {
                for (_, (renderer, transform)) in query {
                    // guard - not enabled
                    if !renderer.form().enabled_in_hierachy() {
                        continue;
                    }

                    // guard - no mesh
                    let Some(asset) = &renderer.asset else {
                        continue;
                    };

                    // add draw call
                    for _ in &asset.mesh {
                        x.draw_calls
                            .push(DrawCall::draw_mesh_single(asset.mesh[0].clone(), asset.materials[0].clone(), transform.get_matrix(), renderer.tint, true));
                    }
                }
            });
        });
    }
}
