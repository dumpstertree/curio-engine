use camera::SysRecordCamera;
use curio_core::{Ledger, Matrix4x4, Nerve, NetworkModes, Quaternion, Vector3};
use ext_rendering::{DrawCall, SysRecordRendering};
use gameplay::{
    built_in::facet::transform::{transform2d::Transform2D, transform3d::Transform3D},
    context_3d::Context3D,
    traits::{habit::Habit, scope::Scope},
    traits_internal::world_context_common::ContextCommon,
};
use habit::habit;

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
        // get cur camera state
        let state_camera = state.read::<SysRecordCamera>();
        let state_time = state.time();

        // edit the state
        state.write::<SysRecordRendering>(|x| {
            // update all instances of the mesh
            world.edit::<&mut RendererDynamic>(|query| {
                for (_, renderer) in query {
                    // update the mesh
                    // if !renderer.get_cached_enabled_in_hierarchy() {
                    //     continue;
                    // }
                    // update all mesh
                    renderer.update_mesh(state_time.scaled_time);
                }
            });

            // push renderers for transform3d
            world.edit::<(&RendererDynamic, &Transform3D)>(|query| {
                for (_, (renderer, transform)) in query {
                    // if !renderer.get_cached_enabled_in_hierarchy() {
                    //     continue;
                    // }

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
            // push renderers for transform2d
            world.edit::<(&mut RendererDynamic, &Transform2D)>(|query| {
                for (_, (renderer, transform)) in query {
                    // if !renderer.get_cached_enabled_in_hierarchy() {
                    //     continue;
                    // }

                    let frustrum_w = 2.1; // these values are made up because the camera is perspective
                    let frustrum_h = 1.1;
                    // guard - no mesh
                    if renderer.asset.is_some() {
                        let zz = 1.0;
                        let xx = remap(transform.position.x, 0.0, 1.0, -frustrum_w / 2.0, frustrum_w / 2.0);
                        let yy = remap(transform.position.y, 1.0, 0.0, -frustrum_h / 2.0, frustrum_h / 2.0);
                        let rotation = state_camera.cameras.rotation * Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0));
                        let position = state_camera.cameras.position + (state_camera.cameras.rotation * Vector3::forward()) * zz + state_camera.cameras.rotation * Vector3::down() * yy + state_camera.cameras.rotation * Vector3::right() * xx;

                        let Some(_asset) = &renderer.asset else {
                            continue;
                        };

                        // add draw call
                        for m in &renderer.mesh {
                            for mesh in &m.mesh {
                                let transform_matrix = Matrix4x4::new(position, rotation, transform.scale);

                                x.draw_calls
                                    .push(DrawCall::draw_mesh_single(mesh.clone(), m.materials[0].clone(), transform_matrix, renderer.tint, false));
                            }
                        }
                    }
                }
            });
        });
    }
}
pub fn remap(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    (value - from_min) / (from_max - from_min) * (to_max - to_min) + to_min
}
