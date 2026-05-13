use crate::{
    built_in::facet::{
        renderer::{renderer_dynamic::RendererDynamic, renderer_image::RendererImage, renderer_static::RendererStatic, renderer_text::RendererText},
        renderer_common::{RendererCommon, update_enabled},
        transform::{
            transform2d::{Transform2D, update_transform2d},
            transform3d::{Transform3D, update_transform3d},
        },
    },
    context_3d::Context3D,
    traits::{habit::Habit, scope::Scope},
    traits_internal::world_context_common::ContextCommon,
};
use curio_core::{
    DrawCall, Matrix4x4, Quaternion, Vector3,
    built_in::record::{sys_record_camera::SysRecordCamera, sys_record_rendering::SysRecordRendering, sys_record_time::SysRecordTime},
    collections::{event_queue::Nerve, ledger::Ledger},
    network_modes::NetworkModes,
};

#[derive(Default)]
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
        let state_camera = state.read::<SysRecordCamera>();

        let time = state.read::<SysRecordTime>().scaled_time;
        //edit draw call states
        update_enabled(world);
        update_transform2d(world);
        update_transform3d(world);
        // for x in world.get::<Renderer>() {
        //     x.update_enabled_in_heirarchy();
        //     x.update_tint_in_heirarchy();
        //     //  (_, (renderer)) in query {
        //     //     renderer.update_enabled_in_heirarchy();
        //     //     renderer.update_tint_in_heirarchy();
        //     // }
        // }

        // world.query_mut::<(&mut RendererAnimated)>(|query| {
        //     for (_, (renderer)) in query {
        //         // renderer.update_enabled_in_heirarchy();
        //         // renderer.update_tint_in_heirarchy();
        //         renderer.update_enabled_in_heirarchy(query)
        //     }
        // });
        // world.query_mut::<(&mut ComponentRendererText)>(|query| {
        //     for (_, (renderer)) in query {
        //         renderer.update_enabled_in_heirarchy();
        //         renderer.update_tint_in_heirarchy();
        //     }
        // });

        state.write::<SysRecordRendering>(|x| {
            // iterate over each renderer

            // for (_entity, (transform, _camera)) in q {
            world.edit::<(&RendererStatic, &Transform2D)>(|query| {
                for (_, (renderer, transform)) in query {
                    // if !renderer.enabled_in_hierarchy(&world) {
                    //     continue;
                    // }

                    let zz = 1.0;
                    let rotation = state_camera.cameras.rotation * Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0));
                    let position = state_camera.cameras.position + (state_camera.cameras.rotation * Vector3::forward()) * zz;

                    // if !renderer.get_cached_enabled_in_hierarchy() {
                    //     continue;
                    // }
                    // guard - no mesh
                    let Some(asset) = &renderer.asset else {
                        continue;
                    };

                    let matrix = Matrix4x4::multiply(&Matrix4x4::new(position, rotation, Vector3::one()), &transform.get_world_matrix(world));

                    // add draw call
                    for _ in &asset.mesh {
                        x.draw_calls
                            .push(DrawCall::draw_mesh_single(asset.mesh[0].clone(), asset.materials[0].clone(), matrix, renderer.get_tint(), false));
                    }
                }
            });
            world.edit::<(&RendererStatic, &Transform3D)>(|query| {
                for (_, (renderer, transform)) in query {
                    // println!( "update {}" ,renderer.asset.clone().unwrap().instance_id);
                    // if !renderer.enabled_in_hierarchy(&world) {
                    //     continue;
                    // }
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
            world.edit::<(&mut RendererDynamic, &Transform3D)>(|query| {
                let mut i = 0;
                for (_, (renderer, _)) in query {
                    // if !renderer.e(&world) {
                    //     continue;
                    // }

                    if !renderer.get_cached_enabled_in_hierarchy() {
                        continue;
                    }

                    // update all mesh
                    renderer.update_mesh(time);
                    i += 1;
                }
            });

            world.edit::<(&mut RendererDynamic, &Transform2D)>(|query| {
                for (_, (renderer, _)) in query {
                    // if !renderer.enabled_in_hierarchy(&world) {
                    //     continue;
                    // }
                    if !renderer.get_cached_enabled_in_hierarchy() {
                        continue;
                    }
                    // update all mesh
                    renderer.update_mesh(time);
                }
            });
            world.edit::<(&RendererDynamic, &Transform3D)>(|query| {
                for (_, (renderer, transform)) in query {
                    // if !renderer.enabled_in_hierarchy(&world) {
                    //     continue;
                    // }
                    if !renderer.get_cached_enabled_in_hierarchy() {
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
                                    .push(DrawCall::draw_mesh_single(mesh.clone(), m.materials[0].clone(), transform.get_matrix(), renderer.get_tint(), true));
                            }
                        }
                    }
                }
            });
            world.edit::<(&mut RendererText, &Transform3D)>(|query| {
                for (_, (renderer, transform)) in query {
                    // if !renderer.enabled_in_hierarchy(&world) {
                    //     continue;
                    // }
                    if !renderer.get_cached_enabled_in_hierarchy() {
                        continue;
                    }
                    renderer.rebuild();
                    for asset_for_matricies in &renderer.asset {
                        for arc_mesh in &asset_for_matricies.0.mesh {
                            let transform_matrix = transform.get_world_matrix(world);
                            let mut inst_matricies = Vec::new();
                            for mesh_matrix in &asset_for_matricies.1 {
                                inst_matricies.push(Matrix4x4::multiply(&transform_matrix, mesh_matrix));
                            }
                            // for inst_matrix in inst_matricies {
                            //     x.draw_calls
                            //         .push(DrawCall::draw_mesh_single(arc_mesh.clone(), asset_for_matricies.0.materials[0].clone(), inst_matrix));
                            // }
                            x.draw_calls
                                .push(DrawCall::draw_mesh_instanced(arc_mesh.clone(), asset_for_matricies.0.materials[0].clone(), inst_matricies, renderer.get_tint(), true));
                        }
                    }
                }
            });

            world.edit::<(&mut RendererDynamic, &Transform2D)>(|query| {
                for (_, (renderer, transform)) in query {
                    // if !renderer.enabled_in_hierarchy(&world) {
                    //     continue;
                    // }
                    if !renderer.get_cached_enabled_in_hierarchy() {
                        continue;
                    }

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
                                    .push(DrawCall::draw_mesh_single(mesh.clone(), m.materials[0].clone(), transform_matrix, renderer.get_tint(), false));
                            }
                        }
                    }
                }
            });
            world.edit::<(&mut RendererText, &Transform2D)>(|query| {
                // let z = 1.0;

                for (_, (renderer, transform)) in query {
                    let zz = 1.0;
                    let rotation = state_camera.cameras.rotation * Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0));
                    let position = state_camera.cameras.position + (state_camera.cameras.rotation * Vector3::forward()) * zz;

                    // if !renderer.enabled_in_hierarchy(&world) {
                    //     continue;
                    // }
                    if !renderer.get_cached_enabled_in_hierarchy() {
                        continue;
                    }

                    renderer.rebuild();
                    for asset_for_matricies in &renderer.asset {
                        for arc_mesh in &asset_for_matricies.0.mesh {
                            // let transform_matrix = Matrix4x4::new(position, rotation, transform.scale);
                            let transform_matrix = Matrix4x4::multiply(&Matrix4x4::new(position, rotation, Vector3::one()), &transform.get_world_matrix(world));
                            let mut inst_matricies = Vec::new();
                            for mesh_matrix in &asset_for_matricies.1 {
                                inst_matricies.push(Matrix4x4::multiply(&transform_matrix, mesh_matrix));
                            }
                            // for inst_matrix in inst_matricies {
                            //     x.draw_calls
                            //         .push(DrawCall::draw_mesh_single(arc_mesh.clone(), asset_for_matricies.0.materials[0].clone(), inst_matrix));
                            // }
                            x.draw_calls
                                .push(DrawCall::draw_mesh_instanced(arc_mesh.clone(), asset_for_matricies.0.materials[0].clone(), inst_matricies, renderer.get_tint(), false));
                        }
                    }
                }
            });
            world.edit::<(&mut RendererImage, &Transform2D)>(|query| {
                for (_, (renderer, transform)) in query {
                    let zz = 1.0;
                    let rotation = state_camera.cameras.rotation * Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0));
                    let position = state_camera.cameras.position + (state_camera.cameras.rotation * Vector3::forward()) * zz;

                    if !renderer.get_cached_enabled_in_hierarchy() {
                        continue;
                    }
                    // guard - no mesh
                    let Some(asset) = &renderer.asset else {
                        continue;
                    };

                    let matrix = Matrix4x4::multiply(&Matrix4x4::new(position, rotation, Vector3::one()), &transform.get_world_matrix(world));
                    let matrix = Matrix4x4::multiply(&matrix, &renderer.bounds_matrix);
                    // add draw call
                    for _ in &asset.mesh {
                        x.draw_calls
                            .push(DrawCall::draw_mesh_single(asset.mesh[0].clone(), asset.materials[0].clone(), matrix, renderer.get_tint(), false));
                    }
                }
            })
        });
    }
}
pub fn remap(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    (value - from_min) / (from_max - from_min) * (to_max - to_min) + to_min
}
