use curio_core::{Ledger, Nerve, NetworkModes};
use ext_rendering::{DrawCall, SysRecordRendering};
use gameplay::{
    built_in::facet::transform::transform3d::Transform3D,
    context_3d::Context3D,
    traits::{habit::Habit, scope::Scope},
    traits_internal::world_context_common::ContextCommon,
};
use habit::habit;
use time::ExtensionsLedger;

use crate::facet::renderer_dynamic::RendererDynamic;

#[habit]
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
        NetworkModes::all()
    }
}
impl Habit for Instance {
    fn did_tick(&mut self, state: &mut Ledger, world: &mut Context3D, _: &mut Nerve) {
        let state_time = state.time();

        // edit the state
        state.write::<SysRecordRendering>(|x| {
            // update all instances of the mesh
            world.edit::<&mut RendererDynamic>(|query| {
                for (_, renderer) in query {
                    // update the mesh
                    renderer.update_mesh(state_time.scaled_time);
                }
            });

            // push renderers for transform3d
            world.edit::<(&RendererDynamic, &Transform3D)>(|query| {
                for (_, (renderer, transform)) in query {
                    // guard - disabled
                    if !renderer.enabled {
                        continue;
                    }

                    // guard - no mesh
                    if renderer.asset.is_some() {
                        let Some(_asset) = &renderer.asset else {
                            continue;
                        };

                        // add draw call
                        for m in &renderer.mesh {
                            for mesh in &m.mesh {
                                x.draw_calls
                                    .push(DrawCall::draw_mesh_single(mesh.clone(), m.materials[0].clone(), transform.get_matrix(), renderer.tint, true));
                            }
                        }
                    }
                }
            });
        });
    }
}
