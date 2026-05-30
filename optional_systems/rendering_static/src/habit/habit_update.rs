use camera::SysRecordCamera;
use curio_core::{Ledger, Matrix4x4, Nerve, NetworkModes, Quaternion, Vector3};
use ext_rendering::{DrawCall, RendererCommon, SysRecordRendering};
use gameplay::{
    built_in::facet::transform::{transform2d::Transform2D, transform3d::Transform3D},
    context_3d::Context3D,
    traits::{habit::Habit, scope::Scope},
    traits_internal::world_context_common::ContextCommon,
};
use habit::habit;

use crate::RendererStatic;

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
        // get cur camera state
        let state_camera = state.read::<SysRecordCamera>();

        // edit the state
        state.write::<SysRecordRendering>(|x| {
            // iterate over each transform2d + renderer
            world.edit::<(&RendererStatic, &Transform2D)>(|query| {
                for (_, (renderer, transform)) in query {
                    // guard - not enabled
                    if !renderer.get_cached_enabled_in_hierarchy() {
                        continue;
                    }

                    // guard - no mesh
                    let Some(asset) = &renderer.asset else {
                        continue;
                    };

                    let zz = 1.0;
                    let rotation = state_camera.cameras.rotation * Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0));
                    let position = state_camera.cameras.position + (state_camera.cameras.rotation * Vector3::forward()) * zz;
                    let matrix = Matrix4x4::multiply(&Matrix4x4::new(position, rotation, Vector3::one()), &transform.get_world_matrix(world));

                    // add draw call
                    for _ in &asset.mesh {
                        x.draw_calls
                            .push(DrawCall::draw_mesh_single(asset.mesh[0].clone(), asset.materials[0].clone(), matrix, renderer.get_tint(), false));
                    }
                }
            });

            // iterate over each transform3d + renderer
            world.edit::<(&RendererStatic, &Transform3D)>(|query| {
                for (_, (renderer, transform)) in query {
                    // guard - not enabled
                    if !renderer.get_cached_enabled_in_hierarchy() {
                        continue;
                    }

                    // guard - no mesh
                    let Some(asset) = &renderer.asset else {
                        continue;
                    };

                    // add draw call
                    for _ in &asset.mesh {
                        x.draw_calls
                            .push(DrawCall::draw_mesh_single(asset.mesh[0].clone(), asset.materials[0].clone(), transform.get_matrix(), renderer.get_tint(), true));
                    }
                }
            });
        });
    }
}
