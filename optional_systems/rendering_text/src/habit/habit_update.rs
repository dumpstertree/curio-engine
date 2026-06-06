use camera::SysRecordCamera;
use curio_core::{Ledger, Matrix4x4, Nerve, NetworkModes, Quaternion, Vector3};
use ext_rendering::{DrawCall, RendererCommon, SysRecordRendering};
use gameplay::{
    built_in::facet::transform::{
        transform2d::{Transform2D, update_transform2d},
        transform3d::{Transform3D, update_transform3d},
    },
    context_3d::Context3D,
    traits::{habit::Habit, scope::Scope},
    traits_internal::world_context_common::ContextCommon,
};
use habit::habit;

use crate::RendererText;

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
    fn did_tick(&mut self, ledger: &mut Ledger, world: &mut Context3D, _: &mut Nerve) {
        ledger.write::<SysRecordRendering>(|x| {
            //
            world.edit::<(&mut RendererText, &Transform3D)>(|query| {
                for (_, (renderer, transform)) in query {
                    //
                    // if !renderer.get_cached_enabled_in_hierarchy() {
                    //     continue;
                    // }

                    renderer.rebuild();
                    for asset_for_matricies in &renderer.asset {
                        for arc_mesh in &asset_for_matricies.0.mesh {
                            let transform_matrix = transform.get_world_matrix(world);
                            let mut inst_matricies = Vec::new();
                            for mesh_matrix in &asset_for_matricies.1 {
                                let m = Matrix4x4::multiply(&transform_matrix, mesh_matrix);
                                inst_matricies.push(m);
                            }

                            x.draw_calls
                                .push(DrawCall::draw_mesh_instanced(arc_mesh.clone(), asset_for_matricies.0.materials[0].clone(), inst_matricies, renderer.get_tint(), true));
                        }
                    }
                }
            })
        });
    }
}
